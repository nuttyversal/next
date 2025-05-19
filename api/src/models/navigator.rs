use argon2::Argon2;
use argon2::PasswordHash;
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::PasswordVerifier;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use chrono::Local;
use chrono::TimeZone;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use thiserror::Error;

use crate::models::NuttyId;
use crate::models::date_time_rfc_3339::DateTimeRfc3339;

/// A registered visitor wandering about in the Nuttyverse.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Navigator {
	#[sqlx(rename = "id")]
	nutty_id: NuttyId,
	name: String,
	#[serde(skip_serializing)]
	pass: String,
	created_at: DateTimeRfc3339,
	updated_at: DateTimeRfc3339,
}

impl Navigator {
	/// Register a new [Navigator] with a securely hashed password.
	pub fn new(name: String, password: &str) -> Result<Self, NavigatorError> {
		Navigator::validate_name(&name)?;

		let salt = SaltString::generate(&mut OsRng);
		let argon2 = Argon2::default();

		let password_hash = argon2
			.hash_password(password.as_bytes(), &salt)
			.map_err(|e| NavigatorError::PasswordHashingError(e.to_string()))?
			.to_string();

		let nutty_id = NuttyId::now();
		let timestamp = nutty_id.timestamp() as i64;

		let now = Local
			.timestamp_millis_opt(timestamp)
			.single()
			.ok_or(NavigatorError::InvalidTimestamp { timestamp })?
			.fixed_offset()
			.into();

		Ok(Self {
			nutty_id,
			name,
			pass: password_hash,
			created_at: now,
			updated_at: now,
		})
	}

	/// Validate a navigator name against format requirements.
	pub fn validate_name(name: &str) -> Result<(), NavigatorError> {
		if name.len() < 4 || name.len() > 16 {
			return Err(NavigatorError::InvalidName(format!(
				"Name must be 4–16 characters (got {})",
				name.len()
			)));
		}

		if !name
			.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
		{
			return Err(NavigatorError::InvalidName(
				"Name must only contain lowercased alphanumeric characters & underscores".to_string(),
			));
		}

		Ok(())
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

	/// Replace the existing name with a new name.
	pub fn update_name(&mut self, new_name: &str) -> Result<(), NavigatorError> {
		Navigator::validate_name(new_name)?;
		self.name = new_name.to_string();

		Ok(())
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

	/// Create a builder for a new navigator.
	pub fn builder() -> NavigatorBuilder {
		NavigatorBuilder::default()
	}

	/// Get the Nutty ID.
	pub fn nutty_id(&self) -> &NuttyId {
		&self.nutty_id
	}

	/// Get the name.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the hashed password.
	pub fn pass(&self) -> &str {
		&self.pass
	}

	/// Get the "created_at" time.
	pub fn created_at(&self) -> &DateTimeRfc3339 {
		&self.created_at
	}

	/// Get the "updated_at" time.
	pub fn updated_at(&self) -> &DateTimeRfc3339 {
		&self.updated_at
	}
}

/// A builder for creating new navigators.
#[derive(Default)]
pub struct NavigatorBuilder {
	nutty_id: Option<NuttyId>,
	name: Option<String>,
	password: Option<String>,
	password_is_hashed: bool,
	created_at: Option<DateTimeRfc3339>,
	updated_at: Option<DateTimeRfc3339>,
}

impl NavigatorBuilder {
	/// Set the Nutty ID.
	pub fn nutty_id(mut self, nutty_id: NuttyId) -> Self {
		self.nutty_id = Some(nutty_id);
		self
	}

	/// Set the navigator name.
	pub fn name(mut self, name: String) -> Self {
		self.name = Some(name);
		self
	}

	/// Set the plaintext password (will be hashed when building).
	pub fn password(mut self, password: String) -> Self {
		self.password = Some(password);
		self
	}

	/// Set a pre-hashed password (will *not* be hashed again when building).
	pub fn password_hash(mut self, hash: String) -> Self {
		self.password = Some(hash);
		self.password_is_hashed = true;
		self
	}

	/// Set the "created at" time.
	pub fn created_at(mut self, created_at: DateTimeRfc3339) -> Self {
		self.created_at = Some(created_at);
		self
	}

	/// Set the "updated at" time.
	pub fn updated_at(mut self, updated_at: DateTimeRfc3339) -> Self {
		self.updated_at = Some(updated_at);
		self
	}

	/// Build the navigator, returning an error if required fields are not set.
	pub fn try_build(self) -> Result<Navigator, NavigatorBuilderError> {
		let name = self.name.ok_or(NavigatorBuilderError::MissingName)?;
		let password = self
			.password
			.ok_or(NavigatorBuilderError::MissingPassword)?;

		match (self.nutty_id, self.created_at, self.updated_at) {
			// Either create the navigator with all timestamps …
			(Some(nutty_id), Some(created_at), Some(updated_at)) => {
				if updated_at < created_at {
					return Err(NavigatorBuilderError::InvalidUpdatedAt);
				}

				let pass = if self.password_is_hashed {
					password
				} else {
					let salt = SaltString::generate(&mut OsRng);
					let argon2 = Argon2::default();

					argon2
						.hash_password(password.as_bytes(), &salt)
						.map_err(|e| NavigatorBuilderError::PasswordHashingError(e.to_string()))?
						.to_string()
				};

				Ok(Navigator {
					nutty_id,
					name,
					pass,
					created_at,
					updated_at,
				})
			}

			// … or with no timestamps at all. Generate them on the fly.
			(None, None, None) => {
				if self.password_is_hashed {
					// If hydrating a navigator that has already been created,
					// then all attributes need to be provided.
					Err(NavigatorBuilderError::MissingTimestampContext)
				} else {
					// If a plaintext password is provided, then that means
					// that we are creating navigator with new Nutty ID.
					Navigator::new(name, &password).map_err(NavigatorBuilderError::CreateNavigator)
				}
			}

			// But, don't create the navigator with partial timestamp context.
			(_, _, _) => Err(NavigatorBuilderError::MissingTimestampContext),
		}
	}
}

#[derive(Debug, Error)]
pub enum NavigatorBuilderError {
	#[error("Name is required")]
	MissingName,

	#[error("Password is required")]
	MissingPassword,

	#[error("Missing timestamp context")]
	MissingTimestampContext,

	#[error("Failed to create navigator: {0}")]
	CreateNavigator(#[from] NavigatorError),

	#[error("Invalid 'updated_at' value: Must be >= 'created_at'")]
	InvalidUpdatedAt,

	#[error("Invalid timestamp from Nutty ID: {timestamp}")]
	InvalidTimestamp { timestamp: i64 },

	#[error("Password hashing failed: {0}")]
	PasswordHashingError(String),
}

#[derive(Debug, Error)]
pub enum NavigatorError {
	#[error("Invalid name format: {0}")]
	InvalidName(String),

	#[error("Invalid timestamp from Nutty ID: {timestamp}")]
	InvalidTimestamp { timestamp: i64 },

	#[error("Password hashing failed: {0}")]
	PasswordHashingError(String),
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_validate_name() {
		// OK if name is valid.
		assert!(Navigator::validate_name("user1").is_ok());
		assert!(Navigator::validate_name("test_user").is_ok());
		assert!(Navigator::validate_name("1234").is_ok()); // Exactly 4 chars.
		assert!(Navigator::validate_name("1234567890123456").is_ok()); // Exactly 16 chars.
		assert!(Navigator::validate_name("a_1_b_2").is_ok());

		// Fail if too short.
		let err = Navigator::validate_name("abc").unwrap_err();
		match err {
			NavigatorError::InvalidName(msg) => {
				assert!(msg.contains("4–16 characters"));
				assert!(msg.contains("(got 3)"));
			}
			_ => panic!("Expected InvalidName error for short name"),
		}

		// Fail if empty string.
		let err = Navigator::validate_name("").unwrap_err();
		match err {
			NavigatorError::InvalidName(msg) => {
				assert!(msg.contains("4–16 characters"));
				assert!(msg.contains("(got 0)"));
			}
			_ => panic!("Expected InvalidName error for empty string"),
		}

		// Fail if too long.
		let err = Navigator::validate_name("12345678901234567").unwrap_err(); // 17 chars.
		match err {
			NavigatorError::InvalidName(msg) => {
				assert!(msg.contains("4–16 characters"));
				assert!(msg.contains("(got 17)"));
			}
			_ => panic!("Expected InvalidName error for long name"),
		}

		// Fail if containing uppercase letters.
		let err = Navigator::validate_name("User1").unwrap_err();
		match err {
			NavigatorError::InvalidName(msg) => {
				assert!(msg.contains("lowercased alphanumeric"));
			}
			_ => panic!("Expected InvalidName error for uppercase letters"),
		}

		// Fail if containing special characters.
		let err = Navigator::validate_name("user-name").unwrap_err();
		match err {
			NavigatorError::InvalidName(msg) => {
				assert!(msg.contains("lowercased alphanumeric"));
			}
			_ => panic!("Expected InvalidName error for special characters"),
		}

		// Fail if containing spaces.
		let err = Navigator::validate_name("user name").unwrap_err();
		match err {
			NavigatorError::InvalidName(msg) => {
				assert!(msg.contains("lowercased alphanumeric"));
			}
			_ => panic!("Expected InvalidName error for space"),
		}

		// Fail if containing Unicode characters.
		let err = Navigator::validate_name("user名").unwrap_err();
		match err {
			NavigatorError::InvalidName(msg) => {
				assert!(msg.contains("lowercased alphanumeric"));
			}
			_ => panic!("Expected InvalidName error for unicode characters"),
		}
	}

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
		let navigator = Navigator::new("special_user".to_string(), special_password).unwrap();
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
		let mut navigator = Navigator::new("invalid_user".to_string(), password).unwrap();

		// Manually corrupt the password hash.
		navigator.pass = "not_a_valid_argon2_hash".to_string();

		// Verification should safely return false rather than panicking.
		assert!(!navigator.verify_password(password));
		assert!(!navigator.verify_password("any_other_password"));
	}

	#[test]
	fn test_navigator_builder() {
		// Create a navigator using the builder.
		let navigator = Navigator::builder()
			.name("test_user".to_string())
			.password("correct horse battery staple".to_string())
			.try_build()
			.unwrap();

		// Verify that the navigator was created correctly.
		assert_eq!(navigator.name(), "test_user");
		assert!(navigator.verify_password("correct horse battery staple"));
	}

	#[test]
	fn test_navigator_builder_with_custom_fields() {
		let nutty_id = NuttyId::now();
		let timestamp = nutty_id.timestamp() as i64;

		let now: DateTimeRfc3339 = Local
			.timestamp_millis_opt(timestamp)
			.single()
			.unwrap()
			.fixed_offset()
			.into();

		let later = (*now.inner() + chrono::Duration::seconds(10)).into();

		// Create a navigator with custom fields.
		let navigator = Navigator::builder()
			.name("custom_user".to_string())
			.password("custom_password".to_string())
			.nutty_id(nutty_id)
			.created_at(now)
			.updated_at(later)
			.try_build()
			.unwrap();

		// Verify that the custom fields were set correctly.
		assert_eq!(navigator.nutty_id(), &nutty_id);
		assert_eq!(navigator.created_at(), &now);
		assert_eq!(navigator.updated_at(), &later);
	}
}
