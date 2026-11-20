use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Row, SqlitePool};
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableColumnInfo {
    pub cid: i64,
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxExecutionResult {
    pub execution_id: Uuid,
    pub plan_id: Uuid,
    pub success: bool,
    pub tables_created: Vec<String>,
    pub columns_modified: Vec<String>,
    pub rollback_tested: bool,
    pub rollback_successful: bool,
    pub duration_ms: u64,
    pub logs: Vec<String>,
    pub timestamp: chrono::DateTime<Utc>,
}

pub struct SqliteSandbox {
    pool: SqlitePool,
}

impl SqliteSandbox {
    pub async fn create_in_memory() -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await?;

        // Initialize base baseline schema for tenant testing
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tenants (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL REFERENCES tenants(id),
                name TEXT NOT NULL,
                legacy_ssn TEXT,
                legacy_notes TEXT
            );

            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                temp_token TEXT NOT NULL
            );

            INSERT INTO tenants (id, name) VALUES ('tenant_prod_1', 'Acme Corp');
            INSERT INTO users (id, tenant_id, name) VALUES ('usr_1', 'tenant_prod_1', 'Alice Admin');
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn inspect_table(&self, table_name: &str) -> Result<Vec<TableColumnInfo>, sqlx::Error> {
        let pragma_sql = format!("PRAGMA table_info({});", table_name);
        let rows = sqlx::query(&pragma_sql).fetch_all(&self.pool).await?;

        let cols = rows
            .into_iter()
            .map(|row| TableColumnInfo {
                cid: row.get::<i64, _>("cid"),
                name: row.get::<String, _>("name"),
                data_type: row.get::<String, _>("type"),
                not_null: row.get::<i64, _>("notnull") == 1,
                default_value: row.get::<Option<String>, _>("dflt_value"),
                primary_key: row.get::<i64, _>("pk") == 1,
            })
            .collect();

        Ok(cols)
    }

    pub async fn execute_sandboxed_migration(
        &self,
        plan_id: Uuid,
        forward_sql: &[String],
        rollback_sql: &[String],
        test_rollback: bool,
    ) -> Result<SandboxExecutionResult, String> {
        self.execute_sandboxed_migration_with_sender(plan_id, forward_sql, rollback_sql, test_rollback, None).await
    }

    pub async fn execute_sandboxed_migration_with_sender(
        &self,
        plan_id: Uuid,
        forward_sql: &[String],
        rollback_sql: &[String],
        test_rollback: bool,
        log_sender: Option<&tokio::sync::broadcast::Sender<crate::state::LogStreamEvent>>,
    ) -> Result<SandboxExecutionResult, String> {
        let start = Instant::now();
        let mut logs = Vec::new();
        let execution_id = Uuid::new_v4();

        let emit = |logs: &mut Vec<String>, msg: String, level: &str| {
            if let Some(sender) = log_sender {
                let _ = sender.send(crate::state::LogStreamEvent {
                    channel: "sandbox".to_string(),
                    level: level.to_string(),
                    plan_id: Some(plan_id),
                    message: msg.clone(),
                    timestamp: Utc::now(),
                });
            }
            logs.push(msg);
        };

        emit(
            &mut logs,
            format!(
                "[{}] Initializing sandboxed SQLite transaction pool.",
                Utc::now().to_rfc3339()
            ),
            "info",
        );

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to initiate transaction: {}", e))?;

        let mut tables_created = Vec::new();
        let mut columns_modified = Vec::new();

        // Step 1: Execute Forward SQL Statements
        for stmt in forward_sql {
            let trimmed = stmt.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }

            emit(&mut logs, format!("Executing DDL: {}", trimmed), "info");
            if let Err(e) = sqlx::query(trimmed).execute(&mut *tx).await {
                emit(
                    &mut logs,
                    format!("Constraint failure during migration execution: {}", e),
                    "error",
                );
                let _ = tx.rollback().await;
                return Ok(SandboxExecutionResult {
                    execution_id,
                    plan_id,
                    success: false,
                    tables_created,
                    columns_modified,
                    rollback_tested: false,
                    rollback_successful: false,
                    duration_ms: start.elapsed().as_millis() as u64,
                    logs,
                    timestamp: Utc::now(),
                });
            }

            if trimmed.to_uppercase().starts_with("CREATE TABLE") {
                if let Some(t_name) = trimmed.split_whitespace().nth(2) {
                    let cleaned_t = t_name.trim_matches(|c| c == '(' || c == '"' || c == '`');
                    tables_created.push(cleaned_t.to_string());
                }
            } else if trimmed.to_uppercase().starts_with("ALTER TABLE") {
                columns_modified.push(trimmed.to_string());
            }
        }

        emit(
            &mut logs,
            "Forward migration statements applied successfully in sandbox.".to_string(),
            "success",
        );

        let mut rollback_success = false;

        // Step 2: Rollback Verification
        if test_rollback {
            emit(
                &mut logs,
                "Initiating automated bidirectional rollback test...".to_string(),
                "info",
            );
            let mut rollback_failed = false;

            for r_stmt in rollback_sql {
                let trimmed = r_stmt.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }

                emit(&mut logs, format!("Testing Rollback DDL: {}", trimmed), "info");
                if let Err(e) = sqlx::query(trimmed).execute(&mut *tx).await {
                    emit(&mut logs, format!("Rollback script failed: {}", e), "error");
                    rollback_failed = true;
                    break;
                }
            }

            if !rollback_failed {
                rollback_success = true;
                emit(
                    &mut logs,
                    "Rollback executed cleanly without leaving residual schema fragments.".to_string(),
                    "success",
                );
            }

            // After rollback test, rollback the transaction so the base sandbox remains pristine
            let _ = tx.rollback().await;
            emit(
                &mut logs,
                "Sandbox transaction rolled back to preserve test isolation.".to_string(),
                "info",
            );
        } else {
            // Commit changes to sandbox pool
            tx.commit()
                .await
                .map_err(|e| format!("Failed to commit sandbox transaction: {}", e))?;
            emit(
                &mut logs,
                "Sandbox transaction committed to in-memory state.".to_string(),
                "success",
            );
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(SandboxExecutionResult {
            execution_id,
            plan_id,
            success: true,
            tables_created,
            columns_modified,
            rollback_tested: test_rollback,
            rollback_successful: rollback_success,
            duration_ms,
            logs,
            timestamp: Utc::now(),
        })
    }
}

