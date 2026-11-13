use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::engine::sqlite_sandbox::SandboxExecutionResult;
use crate::engine::verus_runner::VerusVerificationReport;
use pdd_core::{DecisionCard, MigrationIntent, MigrationPlan};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub intents: Arc<RwLock<HashMap<Uuid, MigrationIntent>>>,
    pub plans: Arc<RwLock<HashMap<Uuid, MigrationPlan>>>,
    pub decisions: Arc<RwLock<HashMap<Uuid, DecisionCard>>>,
    pub verifications: Arc<RwLock<HashMap<Uuid, VerusVerificationReport>>>,
    pub sandbox_executions: Arc<RwLock<HashMap<Uuid, SandboxExecutionResult>>>,
}

impl AppState {
    pub async fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            intents: Arc::new(RwLock::new(HashMap::new())),
            plans: Arc::new(RwLock::new(HashMap::new())),
            decisions: Arc::new(RwLock::new(HashMap::new())),
            verifications: Arc::new(RwLock::new(HashMap::new())),
            sandbox_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

