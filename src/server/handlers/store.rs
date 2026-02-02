use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::server::error::ServerError;
use crate::server::state::AppState;
use crate::service::store as store_service;
use crate::service::types::{QueryResult, RemoveMetricsOptions, WriteMetricRequest};

/// Query parameters for store query
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub filter: Option<String>,
}

/// Query parameters for removing metrics
#[derive(Debug, Deserialize)]
pub struct RemoveParams {
    pub filter: String,
}

/// Write a metric to the store
pub async fn write_metric(
    State(state): State<AppState>,
    Json(request): Json<WriteMetricRequest>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let mut manager = state.manager.write().unwrap();

    let count = store_service::write_metric(&mut manager.store, request)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "total_metrics": count
    })))
}

/// Query metrics from the store
pub async fn query_metrics(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<QueryResult>, ServerError> {
    let mut manager = state.manager.write().unwrap();

    // Load metrics if not already loaded
    store_service::load_metrics(&mut manager.store)?;

    let filter = params.filter.unwrap_or_else(|| ".".to_string());
    let result = store_service::query_metrics(&manager.store, &filter)?;

    Ok(Json(result))
}

/// Remove metrics from the store
pub async fn remove_metrics(
    State(state): State<AppState>,
    Query(params): Query<RemoveParams>,
) -> Result<Json<serde_json::Value>, ServerError> {
    let mut manager = state.manager.write().unwrap();

    // Load metrics if not already loaded
    store_service::load_metrics(&mut manager.store)?;

    let options = RemoveMetricsOptions {
        filter: params.filter,
    };

    let removed_count = store_service::remove_metrics(&mut manager.store, options)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "removed_count": removed_count
    })))
}
