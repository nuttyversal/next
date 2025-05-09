use crate::models::Navigator;
use crate::models::NuttyId;
use crate::models::navigator::NavigatorError;
use crate::models::session::Session;
use crate::models::session::SessionError;
use crate::navigator::repository::NavigatorRepository;
use crate::navigator::repository::NavigatorRepositoryError;

#[derive(Clone)]
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

	/// Login a navigator with their name and password.
	/// Returns a tuple of (Navigator, Session) if successful.
	pub async fn login(
		&self,
		name: String,
		password: String,
		user_agent: String,
	) -> Result<(Navigator, Session), NavigatorServiceError> {
		// Authenticate the navigator.
		let navigator = self
			.repository
			.authenticate(&name, &password)
			.await
			.map_err(NavigatorServiceError::Insert)?
			.ok_or(NavigatorServiceError::InvalidCredentials)?;

		// Create a new session.
		let session = Session::new(*navigator.nutty_id(), user_agent, chrono::Duration::days(1))
			.map_err(NavigatorServiceError::CreateSession)?;

		// Save the session.
		let session = self
			.repository
			.create_session(session)
			.await
			.map_err(NavigatorServiceError::Insert)?;

		Ok((navigator, session))
	}

	/// Get a navigator by ID.
	pub async fn get_navigator_by_id(
		&self,
		id: &NuttyId,
	) -> Result<Option<Navigator>, NavigatorServiceError> {
		self
			.repository
			.get_navigator_by_id(id)
			.await
			.map_err(NavigatorServiceError::Insert)
	}

	/// Get a session by ID.
	pub async fn get_session_by_id(
		&self,
		id: &NuttyId,
	) -> Result<Option<Session>, NavigatorServiceError> {
		self
			.repository
			.get_session_by_id(id)
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

	#[error("Invalid credentials")]
	InvalidCredentials,

	#[error("Failed to create session: {0}")]
	CreateSession(#[from] SessionError),
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
		let service = NavigatorService::new(repo.clone());

		// Act: Register a new navigator.
		let result = service
			.register("test_user".to_string(), "password123".to_string())
			.await;

		// Assert: Verify the registration was successful.
		assert!(result.is_ok());
		let navigator = result.unwrap();
		assert_eq!(navigator.name(), "test_user");
		assert!(navigator.verify_password("password123"));

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
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
		let service = NavigatorService::new(repo.clone());

		// Act: Register a navigator.
		let result_1 = service
			.register("duplicate_user".to_string(), "password123".to_string())
			.await;

		assert!(result_1.is_ok());
		let navigator = result_1.unwrap();

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

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
	}

	#[tokio::test]
	async fn test_login_success() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo.clone());

		// Register a test navigator.
		let navigator = service
			.register("login_test".to_string(), "password123".to_string())
			.await
			.expect("Failed to register test navigator");

		// Act: Login with correct credentials.
		let result = service
			.login(
				"login_test".to_string(),
				"password123".to_string(),
				"test-agent".to_string(),
			)
			.await;

		// Assert: Verify the login was successful.
		assert!(result.is_ok());
		let (logged_in_navigator, session) = result.unwrap();
		assert_eq!(logged_in_navigator.nutty_id(), navigator.nutty_id());
		assert_eq!(session.user_agent(), "test-agent");
		assert!(!session.is_expired());

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
	}

	#[tokio::test]
	async fn test_login_invalid_credentials() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo.clone());

		// Arrange: Register a test navigator.
		let navigator = service
			.register("invalid_test".to_string(), "password123".to_string())
			.await
			.expect("Failed to register test navigator");

		// Act: Try to login with incorrect password.
		let result = service
			.login(
				"invalid_test".to_string(),
				"wrong_password".to_string(),
				"test-agent".to_string(),
			)
			.await;

		// Assert: Verify the error.
		assert!(result.is_err());
		match result.unwrap_err() {
			NavigatorServiceError::InvalidCredentials => (),
			_ => panic!("Expected InvalidCredentials error"),
		}

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
	}

	#[tokio::test]
	async fn test_login_nonexistent_user() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo);

		// Act: Try to login with non-existent user.
		let result = service
			.login(
				"nonexistent".to_string(),
				"password123".to_string(),
				"test-agent".to_string(),
			)
			.await;

		// Assert: Verify the error.
		assert!(result.is_err());
		match result.unwrap_err() {
			NavigatorServiceError::InvalidCredentials => (),
			_ => panic!("Expected InvalidCredentials error"),
		}
	}

	#[tokio::test]
	async fn test_get_navigator_by_id() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo.clone());

		// Register a test navigator.
		let created_navigator = service
			.register("navigator_test".to_string(), "password123".to_string())
			.await
			.expect("Failed to register test navigator");

		// Act: Get the navigator by ID.
		let result = service
			.get_navigator_by_id(created_navigator.nutty_id())
			.await;

		// Assert: Verify the navigator was retrieved successfully.
		assert!(result.is_ok());
		let retrieved_navigator = result.unwrap().expect("Navigator not found");
		assert_eq!(
			*retrieved_navigator.nutty_id(),
			*created_navigator.nutty_id()
		);
		assert_eq!(retrieved_navigator.name(), created_navigator.name());
		assert_eq!(retrieved_navigator.pass(), created_navigator.pass());
		assert_eq!(
			retrieved_navigator.created_at(),
			created_navigator.created_at()
		);
		assert_eq!(
			retrieved_navigator.updated_at(),
			created_navigator.updated_at()
		);

		// Act: Try to get a non-existent navigator.
		let non_existent_id = NuttyId::now();
		let result = service.get_navigator_by_id(&non_existent_id).await;

		// Assert: Verify no navigator was found.
		assert!(result.is_ok());
		assert!(result.unwrap().is_none());

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(created_navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
	}

	#[tokio::test]
	async fn test_get_session_by_id() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);
		let service = NavigatorService::new(repo.clone());

		// Register a test navigator.
		let navigator = service
			.register("session_test".to_string(), "password123".to_string())
			.await
			.expect("Failed to register test navigator");

		// Create a test session.
		let session = Session::new(
			*navigator.nutty_id(),
			"test-agent".to_string(),
			chrono::Duration::days(1),
		)
		.unwrap();

		// Save the session.
		let created_session = repo
			.create_session(session)
			.await
			.expect("Failed to create session");

		// Act: Get the session by ID.
		let result = service.get_session_by_id(created_session.nutty_id()).await;

		// Assert: Verify the session was retrieved successfully.
		assert!(result.is_ok());
		let retrieved_session = result.unwrap().expect("Session not found");
		assert_eq!(*retrieved_session.nutty_id(), *created_session.nutty_id());
		assert_eq!(
			*retrieved_session.navigator_id(),
			*created_session.navigator_id()
		);
		assert_eq!(retrieved_session.user_agent(), created_session.user_agent());
		assert_eq!(retrieved_session.expires_at(), created_session.expires_at());
		assert_eq!(retrieved_session.created_at(), created_session.created_at());
		assert_eq!(retrieved_session.updated_at(), created_session.updated_at());

		// Act: Try to get a non-existent session.
		let non_existent_id = NuttyId::now();
		let result = service.get_session_by_id(&non_existent_id).await;

		// Assert: Verify no session was found.
		assert!(result.is_ok());
		assert!(result.unwrap().is_none());

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete test navigator");
	}
}
