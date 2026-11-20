use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use pdd_core::{process_migration_intent, DecisionCard, PlanStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SubmitIntentRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
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

pub async fn submit_intent(
    State(state): State<AppState>,
    Json(payload): Json<SubmitIntentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (intent, cards, plan) = process_migration_intent(&payload.prompt)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let intent_id = intent.id;
    let plan_id = plan.id;
    let status = plan.status.clone();
    let forward_sql = plan.forward_sql.clone();
    let rollback_sql = plan.rollback_sql.clone();
    let formal_invariants = plan.formal_invariants.clone();
    let summary = plan.summary.clone();

    // Broadcast submission event
    state.emit_log(
        "system",
        "info",
        Some(plan_id),
        format!("Intent submitted: {} action(s) parsed, {} decision card(s) generated.", intent.actions.len(), cards.len()),
    );

    // Persist to state
    {
        let mut intents = state.intents.write().await;
        intents.insert(intent_id, intent);
    }
    {
        let mut decisions = state.decisions.write().await;
        for card in &cards {
            decisions.insert(card.id, card.clone());
        }
    }
    {
        let mut plans = state.plans.write().await;
        plans.insert(plan_id, plan);
    }

    Ok(Json(SubmitIntentResponse {
        intent_id,
        plan_id,
        status,
        decision_cards: cards,
        forward_sql,
        rollback_sql,
        formal_invariants,
        summary,
    }))
}
