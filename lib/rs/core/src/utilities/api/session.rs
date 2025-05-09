use axum::Json;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::models::NuttyId;
use crate::models::navigator::Navigator;
use crate::models::session::Session;
use crate::models::session::SessionError;
use crate::utilities::api::response::Error;
use crate::utilities::api::response::Response;
use crate::utilities::api::state::AppState;

#[derive(Debug, Clone)]
pub struct SessionExtractor {
	pub session: Session,
	pub navigator: Navigator,
}

impl FromRequestParts<AppState> for SessionExtractor {
	type Rejection = (StatusCode, Json<Response<()>>);

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		// Get the session cookie.
		let cookies = parts
			.headers
			.get_all("cookie")
			.iter()
			.filter_map(|v| v.to_str().ok())
			.flat_map(|v| v.split(';'))
			.map(|v| v.trim())
			.collect::<Vec<_>>();

		let session_id = cookies
			.iter()
			.find(|v| v.starts_with("session_id="))
			.and_then(|v| v.strip_prefix("session_id="))
			.ok_or_else(|| {
				let error = Error::from_error(&SessionError::MissingCookie)
					.with_summary("No session cookie found");
				(
					StatusCode::UNAUTHORIZED,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?;

		// Parse the session ID.
		let session_id = Uuid::parse_str(session_id).map_err(|_| {
			let error =
				Error::from_error(&SessionError::InvalidCookie).with_summary("Invalid session cookie");
			(
				StatusCode::UNAUTHORIZED,
				Json(Response::Error {
					errors: vec![error],
				}),
			)
		})?;

		// Get the session from the database.
		let session = state
			.navigator_service
			.get_session_by_id(&NuttyId::new(session_id))
			.await
			.map_err(|e| {
				let error = Error::from_error(&e).with_summary("Failed to retrieve session");
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?
			.ok_or_else(|| {
				let error =
					Error::from_error(&SessionError::SessionNotFound).with_summary("Session not found");
				(
					StatusCode::UNAUTHORIZED,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?;

		// Check if the session is expired.
		if session.is_expired() {
			let error =
				Error::from_error(&SessionError::SessionExpired).with_summary("Session has expired");

			return Err((
				StatusCode::UNAUTHORIZED,
				Json(Response::Error {
					errors: vec![error],
				}),
			));
		}

		// Get the navigator associated with the session.
		let navigator = state
			.navigator_service
			.get_navigator_by_id(session.navigator_id())
			.await
			.map_err(|e| {
				let error = Error::from_error(&e).with_summary("Failed to retrieve navigator");
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?
			.ok_or_else(|| {
				let error = Error::from_error(&SessionError::SessionNotFound)
					.with_summary("Navigator not found");
				(
					StatusCode::UNAUTHORIZED,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?;

		Ok(SessionExtractor { session, navigator })
	}
}
