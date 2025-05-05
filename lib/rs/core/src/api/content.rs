use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::put;

use crate::api::response::Error;
use crate::api::response::Response;
use crate::api::state::AppState;
use crate::models::ContentBlock;
use crate::models::ContentContext;
use crate::models::DissociatedNuttyId;
use crate::models::nutty_id::NuttyIdError;
use crate::services::ContentServiceError;

/// The router for content API endpoints.
pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.route("/content-block/{block_id}", put(content_block_handler))
		.route(
			"/content-block/{block_id}/context",
			get(content_context_handler),
		)
		.with_state(app_state)
}

/// An API handler for fetching the [BlockContext] for a given [ContentBlock].
async fn content_context_handler(
	State(state): State<Arc<AppState>>,
	Path(block_id): Path<String>,
) -> (StatusCode, Json<Response<ContentContext>>) {
	let block_id = DissociatedNuttyId::new(&block_id);

	let block_id = match block_id {
		Ok(id) => id,

		Err(error) => {
			let summary = "Failed to query block context.";
			let error = ContentApiError::LookupBlockContext(error);
			let error = Error::from_error(&error).with_summary(summary);

			return (
				StatusCode::BAD_REQUEST,
				Json(Response::Error {
					errors: vec![error],
				}),
			);
		}
	};

	let block_context = state
		.content_service
		.get_content_block_context(&block_id)
		.await;

	match block_context {
		Ok(block_context) => (
			StatusCode::OK,
			Json(Response::Single {
				data: Some(block_context),
			}),
		),

		Err(error) => {
			let summary = "Failed to query block context.";
			let error = ContentApiError::QueryBlockContext(error);
			let error = Error::from_error(&error).with_summary(summary);

			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(Response::Error {
					errors: vec![error],
				}),
			)
		}
	}
}

/// An API handler for upserting a [ContentBlock].
async fn content_block_handler(
	State(state): State<Arc<AppState>>,
	Path(block_id): Path<String>,
	Json(payload): Json<ContentBlock>,
) -> (StatusCode, Json<Response<ContentBlock>>) {
	// Parse the block ID.
	let block_id = match DissociatedNuttyId::new(&block_id) {
		Ok(id) => id,
		Err(error) => {
			let summary = "Failed to save content block.";
			let error = ContentApiError::LookupBlockContext(error);
			let error = Error::from_error(&error).with_summary(summary);

			return (
				StatusCode::BAD_REQUEST,
				Json(Response::Error {
					errors: vec![error],
				}),
			);
		}
	};

	// Verify the block ID matches the payload.
	if block_id.nid() != payload.nutty_id().nid() {
		let error = ContentApiError::BlockIdMismatch(
			"The block ID in the URL does not match the payload".to_string(),
		);

		let summary = "Failed to save content block.";
		let error = Error::from_error(&error).with_summary(summary);

		return (
			StatusCode::BAD_REQUEST,
			Json(Response::Error {
				errors: vec![error],
			}),
		);
	}

	// Save the content block.
	match state.content_service.save_content_block(payload).await {
		Ok(content_block) => (
			StatusCode::OK,
			Json(Response::Single {
				data: Some(content_block),
			}),
		),

		Err(error) => {
			let summary = "Failed to save content block.";
			let error = ContentApiError::QueryBlockContext(error);
			let error = Error::from_error(&error).with_summary(summary);

			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(Response::Error {
					errors: vec![error],
				}),
			)
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ContentApiError {
	#[error("Unable to look up block context: {0}")]
	LookupBlockContext(#[from] NuttyIdError),

	#[error("Unable to query block context: {0}")]
	QueryBlockContext(#[from] ContentServiceError),

	#[error("Block ID mismatch: {0}")]
	BlockIdMismatch(String),
}
