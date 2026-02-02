use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::error::ServerError;
use crate::server::state::AppState;
use crate::service::workload as wl_service;
use crate::workload::WorkloadMetadata;

/// Request body for adding a workload
#[derive(Debug, Deserialize)]
pub struct AddWorkloadRequest {
    pub language: String,
    pub workload: String,
}

/// Response for adding a workload
#[derive(Debug, Serialize)]
pub struct AddWorkloadResponse {
    pub workload: WorkloadMetadata,
}

/// List workloads in an experiment
pub async fn list_workloads(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<WorkloadMetadata>>, ServerError> {
    let manager = state.manager.read().unwrap();

    let experiment = manager
        .get_experiment(&name)
        .ok_or_else(|| ServerError::not_found(format!("Experiment not found: {}", name)))?;

    let workloads = wl_service::list_workloads(&experiment, None)?;

    Ok(Json(workloads))
}

/// Add a workload to an experiment
pub async fn add_workload(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<AddWorkloadRequest>,
) -> Result<Json<AddWorkloadResponse>, ServerError> {
    let manager = state.manager.read().unwrap();

    let experiment = manager
        .get_experiment(&name)
        .ok_or_else(|| ServerError::not_found(format!("Experiment not found: {}", name)))?;

    let workload =
        wl_service::add_workload(&manager, &experiment, &request.language, &request.workload)?;

    Ok(Json(AddWorkloadResponse { workload }))
}

/// Remove a workload from an experiment
pub async fn remove_workload(
    State(state): State<AppState>,
    Path((name, lang, wl)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let manager = state.manager.read().unwrap();

    let experiment = manager
        .get_experiment(&name)
        .ok_or_else(|| ServerError::not_found(format!("Experiment not found: {}", name)))?;

    wl_service::remove_workload(&experiment, &lang, &wl)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Workload '{}/{}' removed from experiment '{}'", lang, wl, name)
    })))
}
