use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::models::NuttyId;

/// Represents an active [Navigator] login session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
	#[serde(skip_serializing)]
	nutty_id: NuttyId,
	navigator_id: NuttyId,
	#[serde(skip_serializing)]
	user_agent: String,
	expires_at: DateTime<Utc>,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl Session {
	/// Create a new session for a navigator.
	pub fn new(
		navigator_id: NuttyId,
		user_agent: String,
		duration: chrono::Duration,
	) -> Result<Self, SessionError> {
		let nutty_id = NuttyId::now();
		let timestamp = nutty_id.timestamp() as i64;

		let now = Utc
			.timestamp_millis_opt(timestamp)
			.single()
			.ok_or(SessionError::InvalidTimestamp { timestamp })?;

		Ok(Self {
			nutty_id,
			navigator_id,
			user_agent,
			expires_at: now + duration,
			created_at: now,
			updated_at: now,
		})
	}

	/// Check if the session has expired.
	pub fn is_expired(&self) -> bool {
		Utc::now() > self.expires_at
	}

	/// Extend the session's expiration time.
	pub fn extend(&mut self, duration: chrono::Duration) {
		self.expires_at = Utc::now() + duration;
		self.updated_at = Utc::now();
	}

	/// Create a builder for a new session.
	pub fn builder() -> SessionBuilder {
		SessionBuilder::default()
	}

	/// Get the Nutty ID.
	pub fn nutty_id(&self) -> &NuttyId {
		&self.nutty_id
	}

	/// Get the [Navigator] ID.
	pub fn navigator_id(&self) -> &NuttyId {
		&self.navigator_id
	}

	/// Get the user agent.
	pub fn user_agent(&self) -> &str {
		&self.user_agent
	}

	/// Get the expiration time.
	pub fn expires_at(&self) -> &DateTime<Utc> {
		&self.expires_at
	}

	/// Get the creation time.
	pub fn created_at(&self) -> &DateTime<Utc> {
		&self.created_at
	}

	/// Get the last update time.
	pub fn updated_at(&self) -> &DateTime<Utc> {
		&self.updated_at
	}
}

#[derive(Debug, Error)]
pub enum SessionError {
	#[error("Invalid timestamp from Nutty ID: {timestamp}")]
	InvalidTimestamp { timestamp: i64 },

	#[error("Session not found")]
	SessionNotFound,

	#[error("Session expired")]
	SessionExpired,

	#[error("Missing cookie")]
	MissingCookie,

	#[error("Invalid cookie")]
	InvalidCookie,
}

/// A builder for creating new sessions.
#[derive(Default)]
pub struct SessionBuilder {
	nutty_id: Option<NuttyId>,
	navigator_id: Option<NuttyId>,
	user_agent: Option<String>,
	expires_at: Option<DateTime<Utc>>,
	created_at: Option<DateTime<Utc>>,
	updated_at: Option<DateTime<Utc>>,
}

impl SessionBuilder {
	/// Set the Nutty ID.
	pub fn nutty_id(mut self, nutty_id: NuttyId) -> Self {
		self.nutty_id = Some(nutty_id);
		self
	}

	/// Set the navigator ID.
	pub fn navigator_id(mut self, navigator_id: NuttyId) -> Self {
		self.navigator_id = Some(navigator_id);
		self
	}

	/// Set the user agent.
	pub fn user_agent(mut self, user_agent: String) -> Self {
		self.user_agent = Some(user_agent);
		self
	}

	/// Set the expiration time.
	pub fn expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
		self.expires_at = Some(expires_at);
		self
	}

	/// Set the creation time.
	pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
		self.created_at = Some(created_at);
		self
	}

	/// Set the last update time.
	pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
		self.updated_at = Some(updated_at);
		self
	}

	/// Build the session, returning an error if required fields are not set.
	pub fn try_build(self) -> Result<Session, SessionBuilderError> {
		let nutty_id = self.nutty_id.ok_or(SessionBuilderError::MissingNuttyId)?;

		let navigator_id = self
			.navigator_id
			.ok_or(SessionBuilderError::MissingNavigatorId)?;

		let user_agent = self
			.user_agent
			.ok_or(SessionBuilderError::MissingUserAgent)?;

		let expires_at = self
			.expires_at
			.ok_or(SessionBuilderError::MissingExpiresAt)?;

		let created_at = self
			.created_at
			.ok_or(SessionBuilderError::MissingCreatedAt)?;

		let updated_at = self
			.updated_at
			.ok_or(SessionBuilderError::MissingUpdatedAt)?;

		if updated_at < created_at {
			return Err(SessionBuilderError::InvalidUpdatedAt);
		}

		Ok(Session {
			nutty_id,
			navigator_id,
			user_agent,
			expires_at,
			created_at,
			updated_at,
		})
	}
}

#[derive(Debug, Error)]
pub enum SessionBuilderError {
	#[error("Nutty ID is required")]
	MissingNuttyId,

	#[error("Navigator ID is required")]
	MissingNavigatorId,

	#[error("User agent is required")]
	MissingUserAgent,

	#[error("Expiration time is required")]
	MissingExpiresAt,

	#[error("Creation time is required")]
	MissingCreatedAt,

	#[error("Last update time is required")]
	MissingUpdatedAt,

	#[error("Invalid 'updated_at' value: Must be >= 'created_at'")]
	InvalidUpdatedAt,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_session_creation() {
		let navigator_id = NuttyId::now();
		let user_agent = "test-agent".to_string();

		let session =
			Session::new(navigator_id, user_agent.clone(), chrono::Duration::days(30)).unwrap();

		assert_eq!(session.navigator_id(), &navigator_id);
		assert_eq!(session.user_agent(), &user_agent);
		assert!(!session.is_expired());
	}

	#[test]
	fn test_session_expiration() {
		let navigator_id = NuttyId::now();
		let user_agent = "test-agent".to_string();

		let mut session =
			Session::new(navigator_id, user_agent, chrono::Duration::seconds(0)).unwrap();

		// Wait a moment to ensure expiration.
		std::thread::sleep(std::time::Duration::from_millis(100));
		assert!(session.is_expired());

		// Extend the session.
		session.extend(chrono::Duration::days(30));
		assert!(!session.is_expired());
	}

	#[test]
	fn test_session_builder() {
		let nutty_id = NuttyId::now();
		let navigator_id = NuttyId::now();
		let user_agent = "test-agent".to_string();
		let now = Utc::now();
		let expires_at = now + chrono::Duration::days(30);

		let session = Session::builder()
			.nutty_id(nutty_id)
			.navigator_id(navigator_id)
			.user_agent(user_agent)
			.expires_at(expires_at)
			.created_at(now)
			.updated_at(now)
			.try_build()
			.unwrap();

		assert!(!session.is_expired());
	}

	#[test]
	fn test_session_builder_missing_fields() {
		let nutty_id = NuttyId::now();
		let navigator_id = NuttyId::now();
		let user_agent = "test-agent".to_string();
		let now = Utc::now();
		let expires_at = now + chrono::Duration::days(30);

		// Test missing nutty_id.
		let result = Session::builder()
			.navigator_id(navigator_id)
			.user_agent(user_agent.clone())
			.expires_at(expires_at)
			.created_at(now)
			.updated_at(now)
			.try_build();

		assert!(matches!(result, Err(SessionBuilderError::MissingNuttyId)));

		// Test missing navigator_id.
		let result = Session::builder()
			.nutty_id(nutty_id)
			.user_agent(user_agent.clone())
			.expires_at(expires_at)
			.created_at(now)
			.updated_at(now)
			.try_build();

		assert!(matches!(
			result,
			Err(SessionBuilderError::MissingNavigatorId)
		));

		// Test missing user_agent.
		let result = Session::builder()
			.nutty_id(nutty_id)
			.navigator_id(navigator_id)
			.expires_at(expires_at)
			.created_at(now)
			.updated_at(now)
			.try_build();

		assert!(matches!(result, Err(SessionBuilderError::MissingUserAgent)));

		// Test missing expires_at.
		let result = Session::builder()
			.nutty_id(nutty_id)
			.navigator_id(navigator_id)
			.user_agent(user_agent.clone())
			.created_at(now)
			.updated_at(now)
			.try_build();

		assert!(matches!(result, Err(SessionBuilderError::MissingExpiresAt)));

		// Test missing created_at.
		let result = Session::builder()
			.nutty_id(nutty_id)
			.navigator_id(navigator_id)
			.user_agent(user_agent.clone())
			.expires_at(expires_at)
			.updated_at(now)
			.try_build();

		assert!(matches!(result, Err(SessionBuilderError::MissingCreatedAt)));

		// Test missing updated_at.
		let result = Session::builder()
			.nutty_id(nutty_id)
			.navigator_id(navigator_id)
			.user_agent(user_agent)
			.expires_at(expires_at)
			.created_at(now)
			.try_build();

		assert!(matches!(result, Err(SessionBuilderError::MissingUpdatedAt)));
	}

	#[test]
	fn test_session_builder_invalid_timestamps() {
		let nutty_id = NuttyId::now();
		let navigator_id = NuttyId::now();
		let user_agent = "test-agent".to_string();
		let now = Utc::now();
		let earlier = now - chrono::Duration::days(1);

		let result = Session::builder()
			.nutty_id(nutty_id)
			.navigator_id(navigator_id)
			.user_agent(user_agent)
			.expires_at(now + chrono::Duration::days(30))
			.created_at(now)
			.updated_at(earlier)
			.try_build();

		assert!(matches!(result, Err(SessionBuilderError::InvalidUpdatedAt)));
	}
}
