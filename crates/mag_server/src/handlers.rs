use std::path::PathBuf;

use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use mag_core::bin::Bin;

use crate::{
    SharedState,
    errors::ApiError,
    models::{HeatmapData, MetadataUpdate, MethDataFilters, ProjectDetails},
};

#[debug_handler]
pub async fn new_project_handler(
    State(shared_state): State<SharedState>,
    Json(project): Json<ProjectDetails>,
) -> Result<(), ApiError> {
    let mut state = shared_state.lock().unwrap();

    tracing::info!("Creating project: {}", project.project_id.clone());
    state
        .add_project(project)
        .inspect_err(|err| tracing::error!("Failed: {:?}", err))?;
    tracing::info!("Project created successfully");

    Ok(())
}

// pub async fn get_project_contigs(
//     State(shared_state): State<SharedState>,
//     Path(project_id): Path<String>,
// ) -> Result<Json<Vec<String>>, ApiError> {
//     let state = shared_state.lock().unwrap();

//     let project = state
//         .get_project(&project_id)
//         .inspect_err(|err| tracing::error!("Failed to fetch project: {:?}", err))?;

//     let contigs = project.data.get_contigs();
//     Ok(Json(contigs))
// }

pub async fn get_contigs_in_bin(
    State(shared_state): State<SharedState>,
    Path(path_params): Path<(String, String)>,
) -> Result<Json<Vec<String>>, ApiError> {
    let state = shared_state.lock().unwrap();

    let (project_id, bin) = path_params;

    let project = state
        .get_project(&project_id)
        .inspect_err(|err| tracing::error!("Failed to fetch project: {:?}", err))?;

    let filtered = project
        .bins
        .get(&mag_core::bin::BinId(bin.clone()))
        .ok_or_else(|| ApiError::Query(format!("Could not find bin: '{}'", bin)))?
        .contig_metadata
        .iter()
        .map(|c| c.contig_id.0.clone())
        .collect();

    Ok(Json(filtered))
}

#[debug_handler]
pub async fn get_bin_metadata(
    State(shared_state): State<SharedState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<Bin>>, ApiError> {
    let state = shared_state.lock().unwrap();

    let project = state
        .get_project(&project_id)
        .inspect_err(|err| tracing::error!("Failed to fetch project: {:?}", err))?;
    tracing::info!("Fetching bin metadata");

    let bins: Vec<Bin> = project.bins.iter().map(|b| b.1.clone()).collect();

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

    state
        .load_project(path)
        .inspect_err(|err| tracing::error!("Failed: {:?}", err))?;
    tracing::info!("Project loaded successfully");
    Ok(())
}

#[debug_handler]
pub async fn query_heatmap_data(
    State(shared_state): State<SharedState>,
    Path(project_id): Path<String>,
    Json(filters): Json<MethDataFilters>,
) -> Result<Json<HeatmapData>, ApiError> {
    let state = shared_state.lock().unwrap();
    let project = state.get_project(&project_id)?;

    let hm_data = project.get_heatmap_data(filters)?;

    Ok(Json(hm_data))
}

pub async fn update_contig_metadata(
    State(shared_state): State<SharedState>,
    Path(project_id): Path<String>,
    Json(metadata): Json<MetadataUpdate>,
) -> Result<(), ApiError> {
    let mut state = shared_state.lock().unwrap();

    let project = state.get_mut_project(&project_id).inspect_err(|e| {
        tracing::error!("Error finding project'{}': {}", project_id, e.to_string())
    })?;

    project.update_metadata(metadata)?;

    Ok(())
}

pub async fn save_contig_metadata(
    State(shared_state): State<SharedState>,
    Json(project_id): Json<String>,
) -> Result<Json<String>, ApiError> {
    let state = shared_state.lock().unwrap();

    let project = state.get_project(&project_id)?;

    project.save_metadata()?;

    Ok(Json("Metadata saved successfully".to_string()))
}
