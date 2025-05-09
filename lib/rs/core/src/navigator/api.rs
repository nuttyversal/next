use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum_extra::TypedHeader;
use axum_extra::headers::UserAgent;
use cookie::Cookie;
use cookie::SameSite;

use crate::models::Navigator;
use crate::models::session::Session as SessionModel;
use crate::navigator::service::NavigatorServiceError;
use crate::utilities::api::response::Error;
use crate::utilities::api::response::Response;
use crate::utilities::api::session::Session;
use crate::utilities::api::state::AppState;

/// The router for navigator API endpoints.
pub fn router(app_state: Arc<AppState>) -> Router {
	Router::new()
		.route("/navigator", post(register_handler))
		.route("/navigator/login", post(login_handler))
		.route("/navigator/logout", post(logout_handler))
		.route("/navigator/me", get(me_handler))
		.with_state(app_state)
}

/// Request payload for registering a new navigator.
#[derive(serde::Deserialize, serde::Serialize)]
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
			let api_error = NavigatorApiError::Register(error);
			let error_obj = Error::from_error(&api_error);
			let error = error_obj.with_summary(summary);

			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(Response::Error {
					errors: vec![error],
				}),
			)
		}
	}
}

/// Request payload for logging in a navigator.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginRequest {
	name: String,
	pass: String,
}

/// Response payload for a successful login.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
	navigator: Navigator,
	session: SessionModel,
}

/// An API handler for logging in a [Navigator].
#[axum::debug_handler]
async fn login_handler(
	State(state): State<Arc<AppState>>,
	TypedHeader(user_agent): TypedHeader<UserAgent>,
	Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
	match state
		.navigator_service
		.login(payload.name, payload.pass, user_agent.to_string())
		.await
	{
		Ok((navigator, session)) => {
			let cookie = Cookie::build(("session_id", session.nutty_id().to_string()))
				.same_site(SameSite::Strict)
				.secure(true)
				.http_only(true)
				.path("/")
				.max_age(cookie::time::Duration::days(1));

			let cookie_header =
				HeaderValue::from_str(&cookie.to_string()).expect("Failed to create cookie header");

			(
				StatusCode::OK,
				[(SET_COOKIE, cookie_header)],
				Json(Response::Single {
					data: Some(LoginResponse { navigator, session }),
				}),
			)
		}

		Err(error) => {
			let summary = "Failed to login.";
			let api_error = NavigatorApiError::Login(error);
			let error_obj = Error::from_error(&api_error);
			let error = error_obj.with_summary(summary);

			(
				StatusCode::UNAUTHORIZED,
				[(SET_COOKIE, HeaderValue::from_static(""))],
				Json(Response::Error {
					errors: vec![error],
				}),
			)
		}
	}
}

/// An API handler for logging out a [Navigator].
async fn logout_handler(
	State(state): State<Arc<AppState>>,
	Session { session, .. }: Session,
) -> impl IntoResponse {
	match state.navigator_service.logout(session.nutty_id()).await {
		Ok(_) => {
			let expired_cookie = Cookie::build(("session_id", ""))
				.same_site(SameSite::Strict)
				.secure(true)
				.http_only(true)
				.path("/")
				.max_age(cookie::time::Duration::seconds(0));

			let cookie_header = HeaderValue::from_str(&expired_cookie.to_string())
				.expect("Failed to create cookie header");

			(
				StatusCode::OK,
				[(SET_COOKIE, cookie_header)],
				Json(Response::<()>::Single { data: None }),
			)
		}

		Err(error) => {
			let summary = "Failed to logout.";
			let api_error = NavigatorApiError::Logout(error);
			let error_obj = Error::from_error(&api_error);
			let error = error_obj.with_summary(summary);

			(
				StatusCode::INTERNAL_SERVER_ERROR,
				[(SET_COOKIE, HeaderValue::from_static(""))],
				Json(Response::Error {
					errors: vec![error],
				}),
			)
		}
	}
}

/// Response payload for the current navigator's profile.
#[derive(serde::Serialize)]
pub struct MeResponse {
	navigator: Navigator,
}

/// An API handler for getting the current navigator's profile.
async fn me_handler(
	State(_state): State<Arc<AppState>>,
	Session { navigator, .. }: Session,
) -> Json<Response<MeResponse>> {
	// The Session extractor ensures this endpoint is only accessible
	// to authenticated users. If the session is invalid or expired,
	// the request will be rejected before reaching this handler.

	Json(Response::Single {
		data: Some(MeResponse { navigator }),
	})
}

#[derive(Debug, thiserror::Error)]
pub enum NavigatorApiError {
	#[error("Failed to register navigator: {0}")]
	Register(#[from] NavigatorServiceError),

	#[error("Failed to login: {0}")]
	Login(NavigatorServiceError),

	#[error("Failed to logout: {0}")]
	Logout(NavigatorServiceError),
}
