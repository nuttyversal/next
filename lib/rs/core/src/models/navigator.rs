use argon2::Argon2;
use argon2::PasswordHash;
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::PasswordVerifier;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::models::NuttyId;

/// A registered visitor wandering about in the Nuttyverse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Navigator {
	nutty_id: NuttyId,
	name: String,
	pass: String,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl Navigator {
	/// Register a new [Navigator] with a securely hashed password.
	pub fn new(name: String, password: &str) -> Result<Self, NavigatorError> {
		let salt = SaltString::generate(&mut OsRng);
		let argon2 = Argon2::default();

		let password_hash = argon2
			.hash_password(password.as_bytes(), &salt)
			.map_err(|e| NavigatorError::PasswordHashingError(e.to_string()))?
			.to_string();

		let nutty_id = NuttyId::now();
		let timestamp = nutty_id.timestamp() as i64;

		let now = Utc
			.timestamp_millis_opt(timestamp)
			.single()
			.ok_or(NavigatorError::InvalidTimestamp { timestamp })?;

		Ok(Self {
			nutty_id,
			name,
			pass: password_hash,
			created_at: now,
			updated_at: now,
		})
	}

	/// Verify a password attempt against the stored hash.
	pub fn verify_password(&self, password: &str) -> bool {
		let parsed_hash = match PasswordHash::new(&self.pass) {
			Ok(hash) => hash,
			Err(_) => return false,
		};

		Argon2::default()
			.verify_password(password.as_bytes(), &parsed_hash)
			.is_ok()
	}

	/// Replace the existing password with a new password.
	pub fn update_password(&mut self, new_password: &str) -> Result<(), NavigatorError> {
		let salt = SaltString::generate(&mut OsRng);
		let argon2 = Argon2::default();

		self.pass = argon2
			.hash_password(new_password.as_bytes(), &salt)
			.map_err(|e| NavigatorError::PasswordHashingError(e.to_string()))?
			.to_string();

		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum NavigatorError {
	#[error("Password hashing failed: {0}")]
	PasswordHashingError(String),

	#[error("Invalid timestamp from Nutty ID: {timestamp}")]
	InvalidTimestamp { timestamp: i64 },
}
