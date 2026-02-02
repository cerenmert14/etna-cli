use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::error::ServerError;
use crate::server::state::AppState;
use crate::service::experiment as exp_service;
use crate::service::job::JobStatus;
use crate::service::types::{CreateExperimentOptions, ExperimentInfo, RunExperimentOptions};

/// Request body for creating an experiment
#[derive(Debug, Deserialize)]
pub struct CreateExperimentRequest {
    pub name: String,
    pub path: Option<String>,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default)]
    pub register: bool,
    #[serde(default)]
    pub use_local_store: bool,
}

/// Response for creating an experiment
#[derive(Debug, Serialize)]
pub struct CreateExperimentResponse {
    pub experiment: ExperimentInfo,
}

/// Request body for running an experiment
#[derive(Debug, Deserialize)]
pub struct RunExperimentRequest {
    pub tests: Vec<String>,
    #[serde(default)]
    pub short_circuit: bool,
    #[serde(default)]
    pub parallel: bool,
    #[serde(default)]
    pub params: Vec<(String, String)>,
}

/// Response for running an experiment (async job)
#[derive(Debug, Serialize)]
pub struct RunExperimentResponse {
    pub job_id: String,
    pub status: String,
}

/// Request body for visualization
#[derive(Debug, Deserialize)]
pub struct VisualizeRequest {
    pub figure_name: String,
    pub tests: Vec<String>,
    #[serde(default)]
    pub groupby: Vec<String>,
    #[serde(default)]
    pub aggby: Vec<String>,
    #[serde(default = "default_metric")]
    pub metric: String,
    #[serde(default)]
    pub buckets: Vec<f64>,
    pub max: Option<f64>,
    #[serde(default = "default_visualization_type")]
    pub visualization_type: String,
    #[serde(default)]
    pub hatched: Vec<usize>,
}

fn default_metric() -> String {
    "time".to_string()
}

fn default_visualization_type() -> String {
    "bucket".to_string()
}

/// List all experiments
pub async fn list_experiments(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExperimentInfo>>, ServerError> {
    let manager = state.manager.read().unwrap();
    let experiments = exp_service::list_experiments(&manager)?;
    Ok(Json(experiments))
}

/// Get a specific experiment by name
pub async fn get_experiment(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ExperimentInfo>, ServerError> {
    let manager = state.manager.read().unwrap();
    let experiment = exp_service::get_experiment(&manager, &name)?;
    Ok(Json(experiment))
}

/// Create a new experiment
pub async fn create_experiment(
    State(state): State<AppState>,
    Json(request): Json<CreateExperimentRequest>,
) -> Result<Json<CreateExperimentResponse>, ServerError> {
    let mut manager = state.manager.write().unwrap();

    let options = CreateExperimentOptions {
        name: request.name,
        path: request.path.map(PathBuf::from),
        overwrite: request.overwrite,
        register: request.register,
        use_local_store: request.use_local_store,
    };

    let experiment = exp_service::create_experiment(&mut manager, options)?;

    Ok(Json(CreateExperimentResponse { experiment }))
}

/// Delete an experiment
pub async fn delete_experiment(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let mut manager = state.manager.write().unwrap();

    // By default don't delete files - require explicit parameter
    exp_service::delete_experiment(&mut manager, &name, false)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Experiment '{}' deleted", name)
    })))
}

/// Run an experiment (starts async job)
pub async fn run_experiment(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<RunExperimentRequest>,
) -> Result<Json<RunExperimentResponse>, ServerError> {
    // Verify the experiment exists
    {
        let manager = state.manager.read().unwrap();
        let _ = exp_service::get_experiment(&manager, &name)?;
    }

    let options = RunExperimentOptions {
        experiment_name: name.clone(),
        tests: request.tests,
        short_circuit: request.short_circuit,
        parallel: request.parallel,
        params: request.params,
    };

    // Create a job for the experiment run
    let job_id = state.job_manager.create_experiment_run_job(&options)?;

    // Clone what we need for the async task
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    let options_clone = options.clone();

    // Spawn the experiment run in a background task
    tokio::spawn(async move {
        // Update job status to running
        let _ = state_clone
            .job_manager
            .update_job_status(&job_id_clone, JobStatus::Running);

        // Get the manager and run the experiment
        let manager = {
            let mgr = state_clone.manager.read().unwrap();
            // Clone the manager for the sync operation
            // Note: In a real implementation, you'd want a more elegant way to handle this
            crate::manager::Manager {
                experiments: mgr.experiments.clone(),
                store: crate::store::Store::new(mgr.store.path.clone()).unwrap(),
                config: crate::config::EtnaConfig::get_etna_config().unwrap(),
            }
        };

        // Run the experiment synchronously (in the spawned task)
        let result = exp_service::run_experiment(manager, options_clone);

        match result {
            Ok(_) => {
                let _ = state_clone
                    .job_manager
                    .update_job_status(&job_id_clone, JobStatus::Completed);
            }
            Err(e) => {
                let _ = state_clone
                    .job_manager
                    .set_job_error(&job_id_clone, e.to_string());
            }
        }
    });

    Ok(Json(RunExperimentResponse {
        job_id,
        status: "pending".to_string(),
    }))
}

/// Generate visualization (placeholder - synchronous for now)
pub async fn visualize(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<VisualizeRequest>,
) -> Result<Json<serde_json::Value>, ServerError> {
    // Verify the experiment exists
    let experiment = {
        let manager = state.manager.read().unwrap();
        manager
            .get_experiment(&name)
            .ok_or_else(|| ServerError::not_found(format!("Experiment not found: {}", name)))?
    };

    // For now, return a message that this would generate visualization
    // Full implementation would call the visualize service
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!(
            "Visualization '{}' would be generated for experiment '{}'",
            request.figure_name,
            experiment.name
        ),
        "figure_name": request.figure_name,
        "tests": request.tests,
        "visualization_type": request.visualization_type
    })))
}
