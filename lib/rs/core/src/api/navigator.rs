use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;

use crate::api::response::Error;
use crate::api::response::Response;
use crate::api::state::AppState;
use crate::models::Navigator;
use crate::services::NavigatorServiceError;

/// The router for navigator API endpoints.
pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.route("/navigator", post(register_handler))
		.with_state(app_state)
}

/// Request payload for registering a new navigator.
#[derive(serde::Deserialize)]
pub struct RegisterRequest {
	name: String,
	pass: String,
}

/// An API handler for registering a new [Navigator].
async fn register_handler(
	State(state): State<Arc<AppState>>,
	Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<Response<Navigator>>) {
	match state
		.navigator_service
		.register(payload.name, payload.pass)
		.await
	{
		Ok(navigator) => (
			StatusCode::CREATED,
			Json(Response::Single {
				data: Some(navigator),
			}),
		),

		Err(error) => {
			let summary = "Failed to register navigator.";
			let error = NavigatorApiError::Register(error);
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
pub enum NavigatorApiError {
	#[error("Failed to register navigator: {0}")]
	Register(#[from] NavigatorServiceError),
}
