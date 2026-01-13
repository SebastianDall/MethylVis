use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Failed to create a project directory")]
    DirectoryCreation(#[from] std::io::Error),

    #[error("Failed to load data: {0}")]
    DataLoad(#[from] mag_core::errors::DataError),

    #[error("Project id '{0}' Already Exists. Close existing.")]
    ProjectExists(String),

    #[error("Could not find project id '{0}'.")]
    ProjectNotFound(String),

    #[error("Could not create directory toml file: {0}")]
    ProjectTomlError(String),
}

#[derive(Serialize, TS)]
#[ts(export, export_to = "../../../contam-map-frontend/src/bindings/")]
struct ErrorResponse {
    code: u16,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::DirectoryCreation(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
            }
            ApiError::DataLoad(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            ApiError::ProjectExists(msg) => (StatusCode::CONFLICT, msg),
            ApiError::ProjectNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::ProjectTomlError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            code: status.as_u16(),
            message: error_message,
        });

        (status, body).into_response()
    }
}

// impl From<DataError> for ApiError {
//     fn from(value: DataError) -> Self {
//         ApiError::DataLoad(value)
//     }
// }
