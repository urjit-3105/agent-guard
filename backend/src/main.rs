use backend::{create_router, AppState, SqliteSandbox};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Initializing AgentGuard Sandbox Environment...");

    let sandbox = SqliteSandbox::create_in_memory().await?;
    let state = AppState::new(sandbox.pool().clone()).await;

    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("AgentGuard API server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

