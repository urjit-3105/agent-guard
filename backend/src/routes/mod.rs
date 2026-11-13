pub mod decisions;
pub mod intent;
pub mod sandbox;
pub mod verify;

use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/intent/submit", post(intent::submit_intent))
        .route("/api/v1/decisions", get(decisions::list_decisions))
        .route(
            "/api/v1/decisions/:id",
            get(decisions::get_decision),
        )
        .route(
            "/api/v1/decisions/:id/resolve",
            post(decisions::resolve_decision),
        )
        .route("/api/v1/verify", post(verify::run_verify))
        .route("/api/v1/sandbox/execute", post(sandbox::execute_sandbox))
        .layer(cors)
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "agent-guard-backend",
        "version": env!("CARGO_PKG_VERSION"),
        "engine": "Proof-Driven Development (PDD) + Verus Formal SMT",
    }))
}

