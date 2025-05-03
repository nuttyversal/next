use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::extract::State;
use axum::routing::get;

use crate::api::response::Error;
use crate::api::response::Response;
use crate::api::state::AppState;
use crate::models::ContentContext;
use crate::models::DissociatedNuttyId;
use crate::models::nutty_id::NuttyIdError;
use crate::services::ContentServiceError;

/// The router for content API endpoints.
pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.route(
			"/content-block/{block_id}/context",
			get(content_context_handler),
		)
		.with_state(app_state)
}

/// An API handler for fetching the [BlockContext] for a given [ContentBlock].
async fn content_context_handler(
	Path(block_id): Path<String>,
	State(state): State<Arc<AppState>>,
) -> Json<Response<ContentContext>> {
	let block_id = DissociatedNuttyId::new(&block_id);

	let block_id = match block_id {
		Ok(id) => id,

		Err(error) => {
			let summary = "Failed to query block context.";
			let error = ContentApiError::LookupBlockContext(error);
			let error = Error::from_error(&error).with_summary(summary);

			return Json(Response::Error {
				errors: vec![error],
			});
		}
	};

	let block_context = state
		.content_service
		.get_content_block_context(&block_id)
		.await;

	match block_context {
		Ok(block_context) => Json(Response::Single {
			data: Some(block_context),
		}),

		Err(error) => {
			let summary = "Failed to query block context.";
			let error = ContentApiError::QueryBlockContext(error);
			let error = Error::from_error(&error).with_summary(summary);

			Json(Response::Error {
				errors: vec![error],
			})
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ContentApiError {
	#[error("Unable to look up block context: {0}")]
	LookupBlockContext(#[from] NuttyIdError),

	#[error("Unable to query block context: {0}")]
	QueryBlockContext(#[from] ContentServiceError),
}
