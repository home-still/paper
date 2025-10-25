use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaperFetchError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
