use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    SharedState,
    handlers::{get_project_bins, get_projects, load_project, new_project_handler},
};

pub fn create_api_router(state: SharedState) -> Router {
    let api_routes = Router::new()
        .route("/projects", get(get_projects))
        .route("/projects/create", post(new_project_handler))
        .route("/projects/load", post(load_project))
        .route("/projects/{project_id}/bins", get(get_project_bins));

    let router = Router::new().nest("/api", api_routes).with_state(state);

    router
}
