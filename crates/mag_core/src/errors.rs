use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    #[error("Data not found: {0}")]
    NotFound(String),

    #[error("Data assertion error: {0}")]
    DataAssertion(String),
}
