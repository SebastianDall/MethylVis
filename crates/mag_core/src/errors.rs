use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Read error: {0}")]
    Read(#[from] csv::Error),

    #[error("Data not found: {0}")]
    NotFound(String),

    #[error("Data assertion error: {0}")]
    DataAssertion(String),

    #[error("Bin Quality error: {0}")]
    BinQuality(String),
}
