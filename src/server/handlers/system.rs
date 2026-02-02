use axum::{extract::State, Json};
use serde::Deserialize;

use crate::server::error::ServerError;
use crate::server::state::AppState;
use crate::service::check as check_service;
use crate::service::config as config_service;
use crate::service::types::{ConfigInfo, IntegrityCheckOptions, IntegrityCheckResult};

/// Request body for setup
#[derive(Debug, Deserialize)]
pub struct SetupRequest {
    #[serde(default)]
    pub overwrite: bool,
}

/// Request body for integrity check
#[derive(Debug, Deserialize)]
pub struct IntegrityCheckRequest {
    #[serde(default)]
    pub restore: bool,
    #[serde(default)]
    pub remove: bool,
}

/// Get the current configuration
pub async fn get_config() -> Result<Json<ConfigInfo>, ServerError> {
    let config = config_service::get_config()?;
    Ok(Json(config))
}

/// Run the etna setup
pub async fn run_setup(Json(request): Json<SetupRequest>) -> Result<Json<ConfigInfo>, ServerError> {
    let config = config_service::setup(request.overwrite)?;
    Ok(Json(config))
}

/// Run integrity check
pub async fn integrity_check(
    State(state): State<AppState>,
    Json(request): Json<IntegrityCheckRequest>,
) -> Result<Json<IntegrityCheckResult>, ServerError> {
    let mut manager = state.manager.write().unwrap();

    let options = IntegrityCheckOptions {
        restore: request.restore,
        remove: request.remove,
    };

    let result = check_service::integrity_check(&mut manager, options)?;

    Ok(Json(result))
}
