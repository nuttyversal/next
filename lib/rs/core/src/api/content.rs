use crate::{
	api::models::{Error, Response},
	api::state::AppState,
	models::{ContentContext, DissociatedNuttyId},
};
use axum::{
	Json, Router,
	extract::{Path, State},
	routing::get,
};
use std::sync::Arc;

pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.route("/block/{block_id}/context", get(content_context_handler))
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
			let error = Error::from_error(&error).with_summary(summary);

			return Json(Response::Error {
				errors: vec![error],
			});
		}
	};

	let block_context = state
		.content_service
		.get_content_block_context(&block_id.into())
		.await;

	match block_context {
		Ok(block_context) => Json(Response::Single {
			data: Some(block_context),
		}),

		Err(error) => {
			let summary = "Failed to query block context.";
			let error = Error::from_error(&error).with_summary(summary);

			Json(Response::Error {
				errors: vec![error],
			})
		}
	}
}
