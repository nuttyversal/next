use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
	#[error("Database error: {0}")]
	Database(#[from] sqlx::Error),

	#[error("Serialization error: {0}")]
	Serialization(#[from] serde_json::Error),

	#[error("Invalid index: {0}")]
	InvalidIndex(String),
}
