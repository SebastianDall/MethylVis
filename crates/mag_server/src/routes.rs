use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    SharedState,
    handlers::{
        get_contigs_in_bin, get_project_bins, get_project_contigs, get_projects, load_project,
        new_project_handler, query_heatmap_data, save_contig_metadata, update_contig_metadata,
    },
};

pub fn create_api_router(state: SharedState) -> Router {
    let api_routes = Router::new()
        .route("/projects", get(get_projects))
        .route("/projects/create", post(new_project_handler))
        .route("/projects/load", post(load_project))
        .route("/projects/{project_id}/bins", get(get_project_bins))
        .route("/projects/{project_id}/contigs", get(get_project_contigs))
        .route(
            "/projects/{project_id}/contigs/{bin}",
            get(get_contigs_in_bin),
        )
        .route(
            "/projects/{project_id}/data/heatmap",
            post(query_heatmap_data),
        )
        .route("/projects/save", post(save_contig_metadata))
        .route(
            "/projects/{project_id}/data/update",
            post(update_contig_metadata),
        );

    let router = Router::new().nest("/api", api_routes).with_state(state);

    router
}
