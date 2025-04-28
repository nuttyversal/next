use crate::{
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

async fn content_context_handler(
	Path(block_id): Path<String>,
	State(state): State<Arc<AppState>>,
) -> Json<ContentContext> {
	// [TODO] Safely handle unwrapping.
	let block_id = DissociatedNuttyId::new(&block_id).unwrap();

	let block_context = state
		.content_service
		.get_content_block_context(&block_id.into())
		.await;

	// [TODO] Safely handle unwrapping.
	Json(block_context.unwrap())
}
