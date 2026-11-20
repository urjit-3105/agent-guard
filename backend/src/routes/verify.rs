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

    state.emit_log(
        "verus",
        "info",
        Some(plan.id),
        format!("Dispatching Plan {} to Verus SMT / Z3 verification bridge...", plan.id),
    );

    let report = VerusRunner::run_verification(&plan, &intent).await;

    for proof in &report.tenant_isolation_proofs {
        let level = if proof.lemma_satisfied { "success" } else { "error" };
        state.emit_log(
            "verus",
            level,
            Some(plan.id),
            format!("Tenant isolation lemma for table `{}`: {}", proof.table_name, proof.smt_solver_status),
        );
    }

    for proof in &report.schema_safety_proofs {
        let level = if proof.non_destructive_proved { "success" } else { "error" };
        let status = if proof.non_destructive_proved { "PROVED (Non-Destructive)" } else { "VIOLATION DETECTED" };
        state.emit_log(
            "verus",
            level,
            Some(plan.id),
            format!("Schema safety lemma for table `{}`: {}", proof.target_table, status),
        );
    }

    for proof in &report.nullability_proofs {
        let level = if proof.satisfied { "success" } else { "error" };
        let status = if proof.satisfied { "SATISFIED" } else { "VIOLATION (NOT NULL without default)" };
        state.emit_log(
            "verus",
            level,
            Some(plan.id),
            format!("Nullability invariant lemma for `{}.{}`: {}", proof.table_name, proof.column_name, status),
        );
    }

    let report_level = if report.verified { "success" } else { "error" };
    state.emit_log(
        "verus",
        report_level,
        Some(plan.id),
        format!("SMT Solver verdict in {}ms: {}", report.smt_solving_time_ms, report.verdict),
    );

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

