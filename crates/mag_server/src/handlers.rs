use std::path::PathBuf;

use axum::{
    Json, debug_handler,
    extract::{Path, State},
};

use crate::{SharedState, errors::ApiError, models::CreateProjectPayload};

#[debug_handler]
pub async fn new_project_handler(
    State(shared_state): State<SharedState>,
    Json(project): Json<CreateProjectPayload>,
) -> Result<(), ApiError> {
    let mut state = shared_state.lock().unwrap();

    tracing::info!("Creating project: {}", project.project_id.clone());
    state
        .add_project(project)
        .inspect_err(|err| tracing::error!("Failed: {:?}", err))?;
    tracing::info!("Project created successfully");

    Ok(())
}

#[debug_handler]
pub async fn get_project_bins(
    State(shared_state): State<SharedState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<String>>, ApiError> {
    let state = shared_state.lock().unwrap();

    let project = state
        .get_project(&project_id)
        .inspect_err(|err| tracing::error!("Failed to fetch project: {:?}", err))?;

    let bins = project.data.get_bins().inspect_err(|err| {
        tracing::error!("Failed to find bins: {:?}", err);
    })?;

    Ok(Json(bins))
}

#[debug_handler]
pub async fn get_projects(
    State(shared_state): State<SharedState>,
) -> Result<Json<Vec<String>>, ApiError> {
    let state = shared_state.lock().unwrap();

    let projects = state.get_all_project_ids();
    Ok(Json(projects))
}

#[debug_handler]
pub async fn load_project(
    State(shared_state): State<SharedState>,
    Json(path): Json<PathBuf>,
) -> Result<(), ApiError> {
    let mut state = shared_state.lock().unwrap();
    let toml_str =
        std::fs::read_to_string(path).map_err(|e| ApiError::ProjectNotFound(e.to_string()))?;
    let project: CreateProjectPayload =
        toml::from_str(&toml_str).map_err(|e| ApiError::ProjectTomlError(e.to_string()))?;

    tracing::info!("Loading project: {}", project.project_id.clone());
    state
        .load_project(project)
        .inspect_err(|err| tracing::error!("Failed: {:?}", err))?;
    tracing::info!("Project loaded successfully");
    Ok(())
}
