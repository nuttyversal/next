use crate::models::Navigator;
use crate::models::navigator::NavigatorError;
use crate::repository::navigator::NavigatorRepository;
use crate::repository::navigator::NavigatorRepositoryError;

pub struct NavigatorService {
	repository: NavigatorRepository,
}

impl NavigatorService {
	/// Create a new navigator service with the given repository.
	pub fn new(repository: NavigatorRepository) -> Self {
		NavigatorService { repository }
	}

	/// Register a [Navigator].
	pub async fn register(
		&self,
		name: String,
		pass: String,
	) -> Result<Navigator, NavigatorServiceError> {
		let navigator = Navigator::new(name, &pass).map_err(NavigatorServiceError::Create)?;

		self
			.repository
			.create_navigator(navigator)
			.await
			.map_err(NavigatorServiceError::Insert)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum NavigatorServiceError {
	#[error("Failed to create navigator: {0}")]
	Create(#[from] NavigatorError),

	#[error("Failed to create navigator: {0}")]
	Insert(#[from] NavigatorRepositoryError),
}

#[cfg(test)]
mod tests {
	use sqlx::Pool;
	use sqlx::Postgres;
	use sqlx::postgres::PgPoolOptions;

	use super::*;

	async fn connect_to_test_database() -> Pool<Postgres> {
		PgPoolOptions::new()
			.max_connections(5)
			.connect("postgres://nutty@localhost:5432/nuttyverse")
			.await
			.expect("Failed to connect to test database")
	}

	#[tokio::test]
	async fn test_register_success() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo);

		// Act: Register a new navigator.
		let result = service
			.register("test_user".to_string(), "password123".to_string())
			.await;

		// Assert: Verify the registration was successful.
		assert!(result.is_ok());
		let navigator = result.unwrap();
		assert_eq!(navigator.name(), "test_user");
		assert!(navigator.verify_password("password123"));
	}

	#[tokio::test]
	async fn test_register_invalid_name() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo);

		// Act: Try to register with an invalid name.
		let result = service
			.register("a".to_string(), "password123".to_string())
			.await;

		// Assert: Verify the error.
		assert!(result.is_err());
		match result.unwrap_err() {
			NavigatorServiceError::Create(NavigatorError::InvalidName(_)) => (),
			_ => panic!("Expected InvalidName error"),
		}
	}

	#[tokio::test]
	async fn test_register_duplicate_name() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo);

		// Act: Register a navigator.
		let result_1 = service
			.register("duplicate_user".to_string(), "password123".to_string())
			.await;

		assert!(result_1.is_ok());

		// Act: Try to register another navigator with the same name.
		let result_2 = service
			.register("duplicate_user".to_string(), "password456".to_string())
			.await;

		// Assert: Verify the error.
		assert!(result_2.is_err());
		match result_2.unwrap_err() {
			NavigatorServiceError::Insert(NavigatorRepositoryError::QueryFailed(_)) => (),
			_ => panic!("Expected QueryFailed error for duplicate name"),
		}
	}
}
