use std::path::PathBuf;

use axum::{extract::Query, Json};
use serde::Deserialize;

use crate::server::error::ServerError;
use crate::service::mutations as mutation_service;
use crate::service::types::{
    FileMutationsInfo, MutationOperationResponse, ResetMutationsRequest, SetMutationRequest,
};

/// Query parameters for listing mutations
#[derive(Debug, Deserialize)]
pub struct ListMutationsQuery {
    pub path: String,
}

/// Query parameters for getting file mutations
#[derive(Debug, Deserialize)]
pub struct FileMutationsQuery {
    pub path: String,
}

/// List all mutations in a directory
pub async fn list_mutations(
    Query(query): Query<ListMutationsQuery>,
) -> Result<Json<Vec<FileMutationsInfo>>, ServerError> {
    let path = PathBuf::from(&query.path);

    if !path.exists() {
        return Err(ServerError::not_found(format!(
            "Path not found: {}",
            query.path
        )));
    }

    let mutations = mutation_service::list_mutations(&path)?;
    Ok(Json(mutations))
}

/// Get mutations for a specific file with line locations
pub async fn get_file_mutations(
    Query(query): Query<FileMutationsQuery>,
) -> Result<Json<FileMutationsInfo>, ServerError> {
    let path = PathBuf::from(&query.path);

    if !path.exists() {
        return Err(ServerError::not_found(format!(
            "File not found: {}",
            query.path
        )));
    }

    let mutations = mutation_service::get_file_mutations(&path)?;
    Ok(Json(mutations))
}

/// Set a mutation variant as active
pub async fn set_mutation(
    Json(request): Json<SetMutationRequest>,
) -> Result<Json<MutationOperationResponse>, ServerError> {
    if !request.path.exists() {
        return Err(ServerError::not_found(format!(
            "Path not found: {}",
            request.path.display()
        )));
    }

    mutation_service::set_mutation(&request.path, &request.variant, request.glob.as_deref())?;

    Ok(Json(MutationOperationResponse {
        success: true,
        message: format!("Activated mutation variant: {}", request.variant),
    }))
}

/// Reset all mutations in a directory
pub async fn reset_mutations(
    Json(request): Json<ResetMutationsRequest>,
) -> Result<Json<MutationOperationResponse>, ServerError> {
    if !request.path.exists() {
        return Err(ServerError::not_found(format!(
            "Path not found: {}",
            request.path.display()
        )));
    }

    mutation_service::reset_mutations(&request.path)?;

    Ok(Json(MutationOperationResponse {
        success: true,
        message: format!("Reset mutations in: {}", request.path.display()),
    }))
}
