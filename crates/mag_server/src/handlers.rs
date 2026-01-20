use std::path::PathBuf;

use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use axum_extra::extract::Query;
use polars::prelude::*;

use crate::{
    SharedState,
    errors::ApiError,
    models::{BinQueryParams, ContigMetadata, CreateProjectPayload, HeatmapData, MethDataFilters},
};

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
    Query(filter): Query<BinQueryParams>,
) -> Result<Json<Vec<String>>, ApiError> {
    tracing::info!(
        "Fetching bins in project. Filters are: {:#?}",
        filter.quality_filter
    );
    let state = shared_state.lock().unwrap();

    let project = state
        .get_project(&project_id)
        .inspect_err(|err| tracing::error!("Failed to fetch project: {:?}", err))?;

    let quality_filter = if filter.quality_filter.len() == 0 {
        None
    } else {
        Some(
            filter
                .quality_filter
                .into_iter()
                .map(|q| q.to_string())
                .collect::<Vec<String>>(),
        )
    };

    let bins = project.data.get_bins(quality_filter).inspect_err(|err| {
        tracing::error!("Failed to find bins: {:?}", err);
    })?;

    Ok(Json(bins))
}

pub async fn get_project_contigs(
    State(shared_state): State<SharedState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<String>>, ApiError> {
    let state = shared_state.lock().unwrap();

    let project = state
        .get_project(&project_id)
        .inspect_err(|err| tracing::error!("Failed to fetch project: {:?}", err))?;

    let contigs = project.data.get_contigs();
    Ok(Json(contigs))
}

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
        .data
        .contig_bin
        .clone()
        .lazy()
        .filter(col("bin").eq(lit(bin)))
        .collect()?;

    let contig_series = filtered.column("contig")?.str()?;

    let contigs = contig_series
        .into_iter()
        .filter_map(|opt| opt.map(|s| s.to_string()))
        .collect();

    Ok(Json(contigs))
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

    let hm_data = state.get_heatmap_data(&project_id, filters)?;

    Ok(Json(hm_data))
}

pub async fn update_contig_metadata(
    State(shared_state): State<SharedState>,
    Path(project_id): Path<String>,
    Json(contig_metadata): Json<Vec<ContigMetadata>>,
) -> Result<(), ApiError> {
    let mut state = shared_state.lock().unwrap();

    let project = state.get_mut_project(&project_id).inspect_err(|e| {
        tracing::error!("Error finding project'{}': {}", project_id, e.to_string())
    })?;

    project.update_metadata(contig_metadata);

    Ok(())
}

pub async fn save_contig_metadata(
    State(shared_state): State<SharedState>,
    Json(project_id): Json<String>,
) -> Result<(), ApiError> {
    let state = shared_state.lock().unwrap();

    let project = state.get_project(&project_id)?;

    project.save_metadata()?;

    Ok(())
}
