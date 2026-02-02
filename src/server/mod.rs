pub mod error;
pub mod handlers;
pub mod routes;
pub mod state;

use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::manager::Manager;
use crate::service::job::JobManager;

pub use error::ServerError;
pub use state::AppState;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

/// Start the etna HTTP server
pub async fn run_server(config: ServerConfig) -> anyhow::Result<()> {
    // Initialize the manager
    let manager = Manager::load()?;

    // Create the job manager
    let job_manager = JobManager::new();

    // Create the application state
    let state = AppState::new(manager, job_manager);

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        .merge(routes::api_routes())
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // Parse the address
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    tracing::info!("Starting etna server on http://{}", addr);

    // Start the server
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
