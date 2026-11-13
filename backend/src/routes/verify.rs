use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use pdd_core::PlanStatus;
use serde::Deserialize;
use uuid::Uuid;

use crate::engine::VerusRunner;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub plan_id: Uuid,
}

pub async fn run_verify(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (mut plan, intent) = {
        let plans = state.plans.read().await;
        let plan = plans
            .get(&payload.plan_id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Migration plan {} not found", payload.plan_id)))?;

        let intents = state.intents.read().await;
        let intent = intents
            .get(&plan.intent_id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Associated intent not found".to_string()))?;

        (plan, intent)
    };

    let report = VerusRunner::run_verification(&plan, &intent).await;

    // Update plan status based on formal verification outcome
    plan.status = if report.verified {
        PlanStatus::Verified
    } else {
        PlanStatus::Rejected
    };

    {
        let mut plans = state.plans.write().await;
        plans.insert(plan.id, plan);
    }
    {
        let mut verifications = state.verifications.write().await;
        verifications.insert(report.plan_id, report.clone());
    }

    Ok(Json(report))
}

