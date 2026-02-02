use axum::{
    extract::{Path, State},
    Json,
};

use crate::server::error::ServerError;
use crate::server::state::AppState;
use crate::service::job::JobInfo;

/// List all jobs
pub async fn list_jobs(State(state): State<AppState>) -> Result<Json<Vec<JobInfo>>, ServerError> {
    let jobs = state.job_manager.list_jobs()?;
    Ok(Json(jobs))
}

/// Get a specific job by ID
pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JobInfo>, ServerError> {
    let job = state.job_manager.get_job(&id)?;
    Ok(Json(job))
}

/// Cancel a job
pub async fn cancel_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ServerError> {
    state.job_manager.cancel_job(&id)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Job '{}' cancelled", id)
    })))
}
