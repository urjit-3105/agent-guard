use pdd_core::{DecisionCard, PlanStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStage {
    Ingestion,
    PddAnalysis,
    DecisionSignOff,
    VerusProofGate,
    SandboxCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitIntentResponse {
    pub intent_id: Uuid,
    pub plan_id: Uuid,
    pub status: PlanStatus,
    pub decision_cards: Vec<DecisionCard>,
    pub forward_sql: Vec<String>,
    pub rollback_sql: Vec<String>,
    pub formal_invariants: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolveDecisionResponse {
    pub card: DecisionCard,
    pub plan: pdd_core::MigrationPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerusVerificationReport {
    pub plan_id: Uuid,
    pub verified: bool,
    pub formal_proofs_count: usize,
    pub empirical_tests_count: usize,
    pub smt_solver_engine: String,
    pub smt_solving_time_ms: u64,
    pub raw_verus_output: String,
    pub verdict: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogStreamEvent {
    pub channel: String, // "sandbox" | "verus" | "system"
    pub level: String,   // "info" | "warn" | "error" | "success"
    pub plan_id: Option<Uuid>,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

