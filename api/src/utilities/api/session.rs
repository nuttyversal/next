use std::sync::Arc;

use axum::Json;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;

use crate::models::NuttyId;
use crate::models::navigator::Navigator;
use crate::models::session::Session as SessionModel;
use crate::models::session::SessionError;
use crate::utilities::api::response::Error;
use crate::utilities::api::response::Response;
use crate::utilities::api::state::AppState;

#[derive(Debug, Clone)]
pub struct Session {
	pub session: SessionModel,
	pub navigator: Navigator,
}

impl FromRequestParts<Arc<AppState>> for Session {
	type Rejection = (StatusCode, Json<Response<()>>);

	async fn from_request_parts(
		parts: &mut Parts,
		state: &Arc<AppState>,
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
					.with_summary("No session cookie found.");
				(
					StatusCode::UNAUTHORIZED,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?;

		// Parse the session ID as a NuttyId.
		let nutty_id =
			serde_json::from_str::<NuttyId>(&format!("\"{session_id}\"")).map_err(|_| {
				let error = Error::from_error(&SessionError::InvalidCookie)
					.with_summary("Invalid session cookie.");
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
			.get_session_by_id(&nutty_id)
			.await
			.map_err(|e| {
				let error = Error::from_error(&e).with_summary("Failed to retrieve session.");
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?
			.ok_or_else(|| {
				let error =
					Error::from_error(&SessionError::SessionNotFound).with_summary("Session not found.");
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
				Error::from_error(&SessionError::SessionExpired).with_summary("Session has expired.");

			return Err((
				StatusCode::UNAUTHORIZED,
				Json(Response::Error {
					errors: vec![error],
				}),
			));
		}

		// Extract the User-Agent header from the request.
		let request_user_agent = parts
			.headers
			.get("user-agent")
			.and_then(|v| v.to_str().ok())
			.unwrap_or("");

		// Compare User-Agent with the one stored in the session.
		if request_user_agent != session.user_agent() {
			let error = Error::from_error(&SessionError::UserAgentMismatch)
				.with_summary("Detected possible session hijacking attempt.");

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
				let error = Error::from_error(&e).with_summary("Failed to retrieve navigator.");
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?
			.ok_or_else(|| {
				let error = Error::from_error(&SessionError::SessionNotFound)
					.with_summary("Navigator not found.");
				(
					StatusCode::UNAUTHORIZED,
					Json(Response::Error {
						errors: vec![error],
					}),
				)
			})?;

		Ok(Session { session, navigator })
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use axum::body::Body;
	use axum::http::HeaderMap;
	use axum::http::HeaderValue;
	use axum::http::Request;
	use sqlx::Pool;
	use sqlx::Postgres;
	use sqlx::postgres::PgPoolOptions;

	use super::*;
	use crate::content::repository::ContentRepository;
	use crate::content::service::ContentService;
	use crate::navigator::repository::NavigatorRepository;
	use crate::navigator::service::NavigatorService;
	use crate::utilities::api::state::AppState;

	async fn connect_to_test_database() -> Pool<Postgres> {
		let database_url = std::env::var("DATABASE_URL").unwrap();

		PgPoolOptions::new()
			.max_connections(5)
			.connect(&database_url)
			.await
			.expect("Failed to connect to test database")
	}

	#[tokio::test]
	async fn test_session_extractor_from_request_parts() {
		// Arrange: Set up the test dependencies.
		let pool = connect_to_test_database().await;
		let navigator_repository = NavigatorRepository::new(pool.clone());
		let content_repository = ContentRepository::new(pool.clone());
		let navigator_service = NavigatorService::new(navigator_repository.clone());
		let content_service = ContentService::new(content_repository.clone());

		let state = Arc::new(AppState {
			navigator_service,
			content_service,
		});

		// Create a test navigator.
		let navigator = state
			.navigator_service
			.register("test_extractor".to_string(), "password123".to_string())
			.await
			.expect("Failed to register test navigator");

		// Create a test session.
		let session = SessionModel::new(
			*navigator.nutty_id(),
			"test-agent".to_string(),
			chrono::Duration::days(1),
		)
		.unwrap();

		// Save the session.
		let session = navigator_repository
			.create_session(session)
			.await
			.expect("Failed to create session");

		// Create request parts with the session cookie.
		let mut headers = HeaderMap::new();
		let cookie = format!("session_id={}", session.nutty_id());
		headers.insert("cookie", HeaderValue::from_str(&cookie).unwrap());
		headers.insert("user-agent", HeaderValue::from_str("test-agent").unwrap());

		let request = Request::builder()
			.uri("/test")
			.header("cookie", cookie)
			.header("user-agent", "test-agent")
			.body(Body::empty())
			.unwrap();

		let (mut parts, _) = request.into_parts();
		parts.headers = headers;
		parts.extensions.insert(state.clone());

		// Act: Extract the session.
		let result = Session::from_request_parts(&mut parts, &state).await;

		// Assert: Verify successful extraction.
		let extractor = result.unwrap();
		assert_eq!(*extractor.session.nutty_id(), *session.nutty_id());
		assert_eq!(*extractor.navigator.nutty_id(), *navigator.nutty_id());
		assert_eq!(extractor.navigator.name(), "test_extractor");

		// Cleanup.
		navigator_repository
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
	}

	#[tokio::test]
	async fn test_session_extractor_user_agent_validation() {
		// Arrange: Set up the test dependencies.
		let pool = connect_to_test_database().await;
		let navigator_repository = NavigatorRepository::new(pool.clone());
		let content_repository = ContentRepository::new(pool.clone());
		let navigator_service = NavigatorService::new(navigator_repository.clone());
		let content_service = ContentService::new(content_repository.clone());

		let state = Arc::new(AppState {
			navigator_service,
			content_service,
		});

		// Create a test navigator.
		let navigator = state
			.navigator_service
			.register("test_user_agent".to_string(), "password123".to_string())
			.await
			.expect("Failed to register test navigator");

		// Create a test session with a specific User-Agent.
		let specific_user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X)";
		let session = SessionModel::new(
			*navigator.nutty_id(),
			specific_user_agent.to_string(),
			chrono::Duration::days(1),
		)
		.unwrap();

		// Save the session.
		let session = navigator_repository
			.create_session(session)
			.await
			.expect("Failed to create session");

		{
			// Create request with matching User-Agent.
			let mut headers = HeaderMap::new();
			let cookie = format!("session_id={}", session.nutty_id());
			headers.insert("cookie", HeaderValue::from_str(&cookie).unwrap());
			headers.insert(
				"user-agent",
				HeaderValue::from_str(specific_user_agent).unwrap(),
			);

			let request = Request::builder()
				.uri("/test")
				.header("cookie", cookie)
				.header("user-agent", specific_user_agent)
				.body(Body::empty())
				.unwrap();

			let (mut parts, _) = request.into_parts();
			parts.headers = headers;

			// Act: Extract the session.
			let result = Session::from_request_parts(&mut parts, &state).await;

			// Assert: Session is successfully extracted when User-Agent matches.
			assert!(
				result.is_ok(),
				"Session extraction should succeed with matching User-Agent"
			);
			let extractor = result.unwrap();
			assert_eq!(*extractor.session.nutty_id(), *session.nutty_id());
			assert_eq!(extractor.session.user_agent(), specific_user_agent);
		}

		{
			// Create request with different User-Agent.
			let mut headers = HeaderMap::new();
			let cookie = format!("session_id={}", session.nutty_id());
			headers.insert("cookie", HeaderValue::from_str(&cookie).unwrap());
			headers.insert(
				"user-agent",
				HeaderValue::from_str("Different/User-Agent").unwrap(),
			);

			let request = Request::builder()
				.uri("/test")
				.header("cookie", cookie)
				.header("user-agent", "Different/User-Agent")
				.body(Body::empty())
				.unwrap();

			let (mut parts, _) = request.into_parts();
			parts.headers = headers;

			// Act: Extract the session.
			let result = Session::from_request_parts(&mut parts, &state).await;

			// Assert: Session extraction fails with User-Agent mismatch.
			assert!(
				result.is_err(),
				"Session extraction should fail with mismatched User-Agent"
			);

			if let Err((status, response)) = result {
				assert_eq!(status, StatusCode::UNAUTHORIZED);

				if let Response::Error { errors } = response.0 {
					assert!(!errors.is_empty());
				} else {
					panic!("Expected Error response");
				}
			}
		}

		// Cleanup.
		navigator_repository
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
	}
}
