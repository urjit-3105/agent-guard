use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use pdd_core::PlanStatus;
use serde::Deserialize;
use uuid::Uuid;

use crate::engine::SqliteSandbox;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ExecuteSandboxRequest {
    pub plan_id: Uuid,
    pub test_rollback: Option<bool>,
}

pub async fn execute_sandbox(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteSandboxRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut plan = {
        let plans = state.plans.read().await;
        plans
            .get(&payload.plan_id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Plan {} not found", payload.plan_id)))?
    };

    let test_rollback = payload.test_rollback.unwrap_or(true);

    let sandbox = SqliteSandbox::create_in_memory()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create sandbox: {}", e)))?;

    let result = sandbox
        .execute_sandboxed_migration(
            plan.id,
            &plan.forward_sql,
            &plan.rollback_sql,
            test_rollback,
        )
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if result.success && !test_rollback {
        plan.status = PlanStatus::Committed;
        let mut plans = state.plans.write().await;
        plans.insert(plan.id, plan);
    }

    {
        let mut executions = state.sandbox_executions.write().await;
        executions.insert(result.execution_id, result.clone());
    }

    Ok(Json(result))
}

