use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use pdd_core::{DecisionCard, DecisionCardGenerator, MigrationPlan};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ResolveDecisionRequest {
    pub option_id: String,
    pub resolved_by: String,
}

#[derive(Debug, Serialize)]
pub struct ResolveDecisionResponse {
    pub card: DecisionCard,
    pub plan: MigrationPlan,
}

pub async fn list_decisions(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let decisions = state.decisions.read().await;
    let list: Vec<DecisionCard> = decisions.values().cloned().collect();
    Json(list)
}

pub async fn get_decision(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let decisions = state.decisions.read().await;
    let card = decisions
        .get(&id)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Decision card {} not found", id)))?;

    Ok(Json(card))
}

pub async fn resolve_decision(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveDecisionRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Locate card
    let card = {
        let decisions = state.decisions.read().await;
        decisions
            .get(&id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Decision card {} not found", id)))?
    };

    // 2. Locate associated plan and intent
    let (mut plan, intent) = {
        let plans = state.plans.read().await;
        let plan = plans
            .values()
            .find(|p| p.intent_id == card.intent_id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Associated migration plan not found".to_string()))?;

        let intents = state.intents.read().await;
        let intent = intents
            .get(&card.intent_id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Associated intent not found".to_string()))?;

        (plan, intent)
    };

    // 3. Resolve decision card
    DecisionCardGenerator::resolve_decision(
        &mut plan,
        &intent,
        id,
        &payload.option_id,
        &payload.resolved_by,
    )
    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // Find the resolved card
    let resolved_card = plan
        .resolved_decisions
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Card resolution state inconsistency".to_string()))?;

    // 4. Update state
    state.emit_log(
        "system",
        "info",
        Some(plan.id),
        format!("Decision card {} resolved with option: `{}` by `{}`", id, payload.option_id, payload.resolved_by),
    );

    {
        let mut decisions = state.decisions.write().await;
        decisions.insert(id, resolved_card.clone());
    }
    {
        let mut plans = state.plans.write().await;
        plans.insert(plan.id, plan.clone());
    }

    Ok(Json(ResolveDecisionResponse {
        card: resolved_card,
        plan,
    }))
}

