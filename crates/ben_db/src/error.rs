use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("ClickHouse returned non-success: {0}")]
    Server(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for DbError {
    fn from(e: anyhow::Error) -> Self {
        DbError::Other(e.to_string())
    }
}
