use application::HealthService;
use axum::{extract::State, routing::get, Json, Router};
use std::net::SocketAddr;

#[derive(Clone, Default)]
struct AppState {
    health: HealthService,
}

async fn health(State(state): State<AppState>) -> Json<domain::HealthStatus> {
    Json(state.health.get_status())
}

pub async fn run_http(addr: SocketAddr) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/api/health", get(health))
        .with_state(AppState::default());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("HTTP API listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
