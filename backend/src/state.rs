use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::engine::sqlite_sandbox::SandboxExecutionResult;
use crate::engine::verus_runner::VerusVerificationReport;
use pdd_core::{DecisionCard, MigrationIntent, MigrationPlan};
use sqlx::SqlitePool;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogStreamEvent {
    pub channel: String, // "sandbox" | "verus" | "system"
    pub level: String,   // "info" | "warn" | "error" | "success"
    pub plan_id: Option<Uuid>,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub intents: Arc<RwLock<HashMap<Uuid, MigrationIntent>>>,
    pub plans: Arc<RwLock<HashMap<Uuid, MigrationPlan>>>,
    pub decisions: Arc<RwLock<HashMap<Uuid, DecisionCard>>>,
    pub verifications: Arc<RwLock<HashMap<Uuid, VerusVerificationReport>>>,
    pub sandbox_executions: Arc<RwLock<HashMap<Uuid, SandboxExecutionResult>>>,
    pub log_sender: broadcast::Sender<LogStreamEvent>,
}

impl AppState {
    pub async fn new(pool: SqlitePool) -> Self {
        let (log_sender, _) = broadcast::channel(512);
        Self {
            pool,
            intents: Arc::new(RwLock::new(HashMap::new())),
            plans: Arc::new(RwLock::new(HashMap::new())),
            decisions: Arc::new(RwLock::new(HashMap::new())),
            verifications: Arc::new(RwLock::new(HashMap::new())),
            sandbox_executions: Arc::new(RwLock::new(HashMap::new())),
            log_sender,
        }
    }

    pub fn emit_log(
        &self,
        channel: impl Into<String>,
        level: impl Into<String>,
        plan_id: Option<Uuid>,
        message: impl Into<String>,
    ) {
        let _ = self.log_sender.send(LogStreamEvent {
            channel: channel.into(),
            level: level.into(),
            plan_id,
            message: message.into(),
            timestamp: chrono::Utc::now(),
        });
    }
}
