pub mod errors;
pub mod handlers;
pub mod models;
pub mod routes;

use std::sync::{Arc, Mutex};

use crate::{models::AppState, routes::create_api_router};

// create a svelte frotend in appropriate place in repo.
// allow user to create a project
// let user select files to be used. Methylaiton file and contig_bin file
// user should be able to select bin to see methylation of as heatmap. x axis motifs, y axis contigs tile color methylation value
// user should be able to select/deselect motifs to see
// user should be able to mark contigs in a bin which will mark these contigs as contamination in the project file created.

type SharedState = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let state = AppState::new();
    let shared_state: SharedState = Arc::new(Mutex::new(state));

    // let app = Router::new().route("/", get(root)).with_state(shared_state);
    let app = create_api_router(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
