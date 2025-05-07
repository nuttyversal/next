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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_password_verification() {
		// Create a new navigator with a known password.
		let password = "correct horse battery staple";
		let navigator = Navigator::new("test_user".to_string(), password).unwrap();

		// Verify that the correct password works.
		assert!(navigator.verify_password(password));

		// Verify that incorrect passwords fail.
		assert!(!navigator.verify_password("wrong_password"));
		assert!(!navigator.verify_password(""));
		assert!(!navigator.verify_password("CORRECT HORSE BATTERY STAPLE")); // Case sensitivity.
		assert!(!navigator.verify_password(&password[0..password.len() - 1])); // Partial password.
	}

	#[test]
	fn test_password_update() {
		// Create a new navigator with an initial password.
		let initial_password = "initial_password";
		let mut navigator = Navigator::new("test_user".to_string(), initial_password).unwrap();

		// Verify that the initial password works.
		assert!(navigator.verify_password(initial_password));

		// Update the password.
		let new_password = "new_password";
		navigator.update_password(new_password).unwrap();

		// Verify the new password works.
		assert!(navigator.verify_password(new_password));

		// Verify the old password no longer works.
		assert!(!navigator.verify_password(initial_password));
	}

	#[test]
	fn test_password_edge_cases() {
		// Test with an empty password (should work for creation, but is not recommended).
		let empty_password = "";
		let navigator = Navigator::new("empty_pass_user".to_string(), empty_password).unwrap();
		assert!(navigator.verify_password(empty_password));
		assert!(!navigator.verify_password("not_empty"));

		// Test with a long password.
		let long_password = "a".repeat(100);
		let navigator = Navigator::new("long_pass_user".to_string(), &long_password).unwrap();
		assert!(navigator.verify_password(&long_password));
		assert!(!navigator.verify_password(&"a".repeat(99)));

		// Test with special characters.
		let special_password = "!@#$%^&*()_+{}|:<>?~";
		let navigator = Navigator::new("special_char_user".to_string(), special_password).unwrap();
		assert!(navigator.verify_password(special_password));
		assert!(!navigator.verify_password("wrong"));
	}

	#[test]
	fn test_sequential_password_updates() {
		// Create a navigator, and update the password multiple times.
		let password_1 = "password_1";
		let mut navigator = Navigator::new("update_user".to_string(), password_1).unwrap();

		// Make the first update.
		let password_2 = "password_2";
		navigator.update_password(password_2).unwrap();
		assert!(navigator.verify_password(password_2));
		assert!(!navigator.verify_password(password_1));

		// Make the second update.
		let password_3 = "password_3";
		navigator.update_password(password_3).unwrap();
		assert!(navigator.verify_password(password_3));
		assert!(!navigator.verify_password(password_2));
		assert!(!navigator.verify_password(password_1));
	}

	#[test]
	fn test_invalid_password_hash() {
		// Create a navigator with a valid password.
		let password = "valid_password";
		let mut navigator = Navigator::new("InvalidHashUser".to_string(), password).unwrap();

		// Manually corrupt the password hash.
		navigator.pass = "not_a_valid_argon2_hash".to_string();

		// Verification should safely return false rather than panicking.
		assert!(!navigator.verify_password(password));
		assert!(!navigator.verify_password("any_other_password"));
	}
}
