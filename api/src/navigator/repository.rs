use sqlx::Executor;
use sqlx::Postgres;
use thiserror::Error;

use crate::models::Navigator;
use crate::models::NuttyId;
use crate::models::navigator::NavigatorBuilderError;
use crate::models::navigator::NavigatorError;
use crate::models::session::Session;
use crate::models::session::SessionBuilderError;

/// A repository for navigator accounts.
/// Objects are stored in PostgreSQL.
#[derive(Debug, Clone)]
pub struct NavigatorRepository {
	/// The PostgreSQL database pool.
	pool: sqlx::Pool<Postgres>,
}

impl NavigatorRepository {
	/// Create a new navigator repository.
	pub fn new(pool: sqlx::Pool<Postgres>) -> Self {
		Self { pool }
	}

	/// Create a new navigator.
	pub async fn create_navigator_tx<'e, E>(
		&self,
		executor: E,
		navigator: Navigator,
	) -> Result<Navigator, NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		Ok(sqlx::query_as(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, $4, $5, $6)
				RETURNING id, name, pass, created_at, updated_at
			"#,
		)
		.bind(navigator.nutty_id().uuid())
		.bind(navigator.nutty_id().nid())
		.bind(navigator.name())
		.bind(navigator.pass())
		.bind(navigator.created_at())
		.bind(navigator.updated_at())
		.fetch_one(executor)
		.await?)
	}

	/// Create a new navigator.
	pub async fn create_navigator(
		&self,
		navigator: Navigator,
	) -> Result<Navigator, NavigatorRepositoryError> {
		self.create_navigator_tx(&self.pool, navigator).await
	}

	/// Get a navigator by ID.
	pub async fn get_navigator_by_id_tx<'e, E>(
		&self,
		executor: E,
		id: &NuttyId,
	) -> Result<Option<Navigator>, NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		Ok(sqlx::query_as(
			r#"
				SELECT id, name, pass, created_at, updated_at
				FROM auth.navigators
				WHERE id = $1
			"#,
		)
		.bind(id.uuid())
		.fetch_optional(executor)
		.await?)
	}

	/// Get a navigator by ID.
	pub async fn get_navigator_by_id(
		&self,
		id: &NuttyId,
	) -> Result<Option<Navigator>, NavigatorRepositoryError> {
		self.get_navigator_by_id_tx(&self.pool, id).await
	}

	/// Get a navigator by name.
	pub async fn get_navigator_by_name_tx<'e, E>(
		&self,
		executor: E,
		name: &str,
	) -> Result<Option<Navigator>, NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		Ok(sqlx::query_as(
			r#"
				SELECT id, name, pass, created_at, updated_at
				FROM auth.navigators
				WHERE name = $1
			"#,
		)
		.bind(name)
		.fetch_optional(executor)
		.await?)
	}

	/// Get a navigator by name.
	pub async fn get_navigator_by_name(
		&self,
		name: &str,
	) -> Result<Option<Navigator>, NavigatorRepositoryError> {
		self.get_navigator_by_name_tx(&self.pool, name).await
	}

	/// Update a navigator account.
	pub async fn update_navigator_tx<'e, E>(
		&self,
		executor: E,
		navigator: Navigator,
	) -> Result<Navigator, NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		// Update the navigator record.
		Ok(sqlx::query_as(
			r#"
				UPDATE auth.navigators
				SET name = $2, pass = $3
				WHERE id = $1
				RETURNING id, name, pass, created_at, updated_at
			"#,
		)
		.bind(navigator.nutty_id().uuid())
		.bind(navigator.name())
		.bind(navigator.pass())
		.fetch_one(executor)
		.await?)
	}

	/// Update a navigator account.
	pub async fn update_navigator(
		&self,
		navigator: Navigator,
	) -> Result<Navigator, NavigatorRepositoryError> {
		self.update_navigator_tx(&self.pool, navigator).await
	}

	/// Update a navigator's password.
	pub async fn update_password_tx<'e, E>(
		&self,
		executor: E,
		id: &NuttyId,
		new_password: &str,
	) -> Result<(), NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres> + Clone,
	{
		// Find the navigator.
		let navigator = self
			.get_navigator_by_id_tx(executor.clone(), id)
			.await?
			.ok_or(NavigatorRepositoryError::NavigatorNotFound)?
			.clone();

		// Update the password.
		let mut navigator = navigator.clone();
		navigator
			.update_password(new_password)
			.map_err(NavigatorRepositoryError::ModelError)?;

		// Save the updated navigator.
		self.update_navigator_tx(executor, navigator).await?;

		Ok(())
	}

	/// Update a navigator's password.
	pub async fn update_password(
		&self,
		id: &NuttyId,
		new_password: &str,
	) -> Result<(), NavigatorRepositoryError> {
		self.update_password_tx(&self.pool, id, new_password).await
	}

	/// Delete a navigator account.
	pub async fn delete_navigator_tx<'e, E>(
		&self,
		executor: E,
		id: &NuttyId,
	) -> Result<(), NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		sqlx::query!(
			r#"
				DELETE FROM auth.navigators
				WHERE id = $1
			"#,
			id.uuid(),
		)
		.execute(executor)
		.await?;

		Ok(())
	}

	/// Delete a navigator account.
	pub async fn delete_navigator(&self, id: &NuttyId) -> Result<(), NavigatorRepositoryError> {
		self.delete_navigator_tx(&self.pool, id).await
	}

	/// Authenticate a navigator with name and password.
	pub async fn authenticate_tx<'e, E>(
		&self,
		executor: E,
		name: &str,
		password: &str,
	) -> Result<Option<Navigator>, NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		// Find the navigator by name.
		let navigator = match self.get_navigator_by_name_tx(executor, name).await? {
			Some(navigator) => navigator,
			None => return Ok(None),
		};

		// Verify the password.
		if navigator.verify_password(password) {
			Ok(Some(navigator))
		} else {
			Ok(None)
		}
	}

	/// Authenticate a navigator with name and password.
	pub async fn authenticate(
		&self,
		name: &str,
		password: &str,
	) -> Result<Option<Navigator>, NavigatorRepositoryError> {
		self.authenticate_tx(&self.pool, name, password).await
	}

	/// Create a new session for a navigator.
	pub async fn create_session_tx<'e, E>(
		&self,
		executor: E,
		session: Session,
	) -> Result<Session, NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		Ok(sqlx::query_as(
			r#"
				INSERT INTO auth.sessions (id, nutty_id, navigator_id, user_agent, expires_at, created_at, updated_at)
				VALUES ($1, $2, $3, $4, $5, $6, $7)
				RETURNING id, navigator_id, user_agent, expires_at, created_at, updated_at
			"#,
		)
			.bind(session.nutty_id().uuid())
			.bind(session.nutty_id().nid())
			.bind(session.navigator_id().uuid())
			.bind(session.user_agent())
			.bind(session.expires_at())
			.bind(session.created_at())
			.bind(session.updated_at())
		.fetch_one(executor)
		.await?)
	}

	/// Create a new session for a navigator.
	pub async fn create_session(
		&self,
		session: Session,
	) -> Result<Session, NavigatorRepositoryError> {
		self.create_session_tx(&self.pool, session).await
	}

	/// Get a session by ID.
	pub async fn get_session_by_id_tx<'e, E>(
		&self,
		executor: E,
		id: &NuttyId,
	) -> Result<Option<Session>, NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		Ok(sqlx::query_as(
			r#"
				SELECT id, navigator_id, user_agent, expires_at, created_at, updated_at
				FROM auth.sessions
				WHERE id = $1
			"#,
		)
		.bind(id.uuid())
		.fetch_optional(executor)
		.await?)
	}

	/// Get a session by ID.
	pub async fn get_session_by_id(
		&self,
		id: &NuttyId,
	) -> Result<Option<Session>, NavigatorRepositoryError> {
		self.get_session_by_id_tx(&self.pool, id).await
	}

	/// Delete a session by ID.
	pub async fn delete_session_tx<'e, E>(
		&self,
		executor: E,
		id: &NuttyId,
	) -> Result<(), NavigatorRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		sqlx::query!(
			r#"
				DELETE FROM auth.sessions
				WHERE id = $1
			"#,
			id.uuid(),
		)
		.execute(executor)
		.await?;

		Ok(())
	}

	/// Delete a session by ID.
	pub async fn delete_session(&self, id: &NuttyId) -> Result<(), NavigatorRepositoryError> {
		self.delete_session_tx(&self.pool, id).await
	}
}

#[derive(Debug, Error)]
pub enum NavigatorRepositoryError {
	#[error("Database query failed: {0}")]
	QueryFailed(#[from] sqlx::error::Error),

	#[error("Navigator model error: {0}")]
	ModelError(#[from] NavigatorError),

	#[error("Navigator builder error: {0}")]
	BuilderError(#[from] NavigatorBuilderError),

	#[error("Session builder error: {0}")]
	SessionBuilderError(#[from] SessionBuilderError),

	#[error("Navigator not found")]
	NavigatorNotFound,
}

#[cfg(test)]
mod tests {
	use sqlx::Pool;
	use sqlx::Postgres;
	use sqlx::postgres::PgPoolOptions;

	use super::*;
	use crate::models::Navigator;
	use crate::models::session::Session;

	async fn connect_to_test_database() -> Pool<Postgres> {
		PgPoolOptions::new()
			.max_connections(5)
			.connect("postgres://nutty@localhost:5432/nuttyverse")
			.await
			.expect("Failed to connect to test database")
	}

	#[tokio::test]
	async fn test_navigator_crud_operations() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);

		// Arrange: Create a test navigator.
		let name = "test_user";
		let password = "test_password";
		let navigator = Navigator::new(name.to_string(), password).unwrap();
		let id = navigator.nutty_id();

		// Act: Save the navigator.
		let saved_navigator = repo
			.create_navigator(navigator.clone())
			.await
			.expect("Failed to create navigator");

		// Assert: The saved navigator matches the original.
		assert_eq!(*saved_navigator.nutty_id(), *id);
		assert_eq!(saved_navigator.name(), name);

		// Act: Get the navigator by ID.
		let retrieved_by_id = repo
			.get_navigator_by_id(id)
			.await
			.expect("Failed to get navigator by ID")
			.expect("Navigator not found by ID");

		// Assert: The retrieved navigator matches the original.
		assert_eq!(*retrieved_by_id.nutty_id(), *id);
		assert_eq!(retrieved_by_id.name(), name);

		// Act: Get the navigator by name.
		let retrieved_by_name = repo
			.get_navigator_by_name(name)
			.await
			.expect("Failed to get navigator by name")
			.expect("Navigator not found by name");

		// Assert: The retrieved navigator matches the original.
		assert_eq!(*retrieved_by_name.nutty_id(), *id);
		assert_eq!(retrieved_by_name.name(), name);

		// Act: Update the navigator.
		let mut updated_navigator = retrieved_by_id.clone();
		let _ = updated_navigator.update_name("updated_user");

		let updated = repo
			.update_navigator(updated_navigator)
			.await
			.expect("Failed to update navigator");

		// Assert: The updated navigator has the new name.
		assert_eq!(*updated.nutty_id(), *id);
		assert_eq!(updated.name(), "updated_user");

		// Act: Update the password.
		let new_password = "new_password";
		repo
			.update_password(id, new_password)
			.await
			.expect("Failed to update password");

		// Assert: The password was updated.
		let authenticated = repo
			.authenticate("updated_user", new_password)
			.await
			.expect("Failed to authenticate")
			.expect("Authentication failed");

		assert_eq!(*authenticated.nutty_id(), *id);

		// Act: Try to authenticate with old password.
		let invalid_auth = repo
			.authenticate("updated_user", password)
			.await
			.expect("Failed to attempt authentication");

		assert!(invalid_auth.is_none());

		// Act: Delete the navigator.
		repo
			.delete_navigator(id)
			.await
			.expect("Failed to delete navigator");

		// Assert: The navigator no longer exists.
		let deleted_check = repo
			.get_navigator_by_id(id)
			.await
			.expect("Failed to check for deleted navigator");

		assert!(deleted_check.is_none());
	}

	#[tokio::test]
	async fn test_authenticate() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);

		// Arrange: Create a test navigator.
		let name = "auth_test_user";
		let password = "test_password";
		let navigator = Navigator::new(name.to_string(), password).unwrap();

		// Act: Save the navigator.
		repo
			.create_navigator(navigator.clone())
			.await
			.expect("Failed to create navigator");

		// Act & Assert: Successful authentication.
		let auth_result = repo
			.authenticate(name, password)
			.await
			.expect("Failed to authenticate");

		assert!(auth_result.is_some());

		// Act & Assert: Failed authentication - wrong password.
		let wrong_password = repo
			.authenticate(name, "wrong_password")
			.await
			.expect("Failed to attempt authentication");

		assert!(wrong_password.is_none());

		// Act & Assert: Failed authentication - wrong username.
		let wrong_username = repo
			.authenticate("wrong_username", password)
			.await
			.expect("Failed to attempt authentication");

		assert!(wrong_username.is_none());

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete navigator");
	}

	#[tokio::test]
	async fn test_create_session() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);

		// Arrange: Create a test navigator.
		let name = "session_test";
		let password = "test_password";
		let navigator = Navigator::new(name.to_string(), password).unwrap();
		let navigator = repo
			.create_navigator(navigator)
			.await
			.expect("Failed to create navigator");

		// Arrange: Create a test session.
		let user_agent = "test-agent".to_string();
		let session = Session::new(
			*navigator.nutty_id(),
			user_agent.clone(),
			chrono::Duration::days(30),
		)
		.unwrap();

		// Act: Create the session.
		let created_session = repo
			.create_session(session.clone())
			.await
			.expect("Failed to create session");

		// Assert: The created session matches the original.
		assert_eq!(*created_session.nutty_id(), *session.nutty_id());
		assert_eq!(*created_session.navigator_id(), *session.navigator_id());
		assert_eq!(created_session.user_agent(), session.user_agent());
		assert_eq!(created_session.expires_at(), session.expires_at());
		assert_eq!(created_session.created_at(), session.created_at());
		assert_eq!(created_session.updated_at(), session.updated_at());

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete navigator");
	}

	#[tokio::test]
	async fn test_get_session_by_id() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);

		// Arrange: Create a test navigator.
		let name = "session_test";
		let password = "test_password";
		let navigator = Navigator::new(name.to_string(), password).unwrap();

		let navigator = repo
			.create_navigator(navigator)
			.await
			.expect("Failed to create navigator");

		// Arrange: Create a test session.
		let user_agent = "test-agent".to_string();
		let session = Session::new(
			*navigator.nutty_id(),
			user_agent.clone(),
			chrono::Duration::days(30),
		)
		.unwrap();

		// Act: Create the session.
		let created_session = repo
			.create_session(session.clone())
			.await
			.expect("Failed to create session");

		// Act: Get the session by ID.
		let retrieved_session = repo
			.get_session_by_id(created_session.nutty_id())
			.await
			.expect("Failed to get session by ID")
			.expect("Session not found");

		// Assert: The retrieved session matches the original.
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
		let not_found_session = repo
			.get_session_by_id(&non_existent_id)
			.await
			.expect("Failed to check for non-existent session");

		// Assert: No session was found.
		assert!(not_found_session.is_none());

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete navigator");
	}

	#[tokio::test]
	async fn test_delete_session() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = NavigatorRepository::new(pool);

		// Arrange: Create a test navigator.
		let name = "delete_test";
		let password = "test_password";
		let navigator = Navigator::new(name.to_string(), password).unwrap();
		let navigator = repo
			.create_navigator(navigator)
			.await
			.expect("Failed to create navigator");

		// Arrange: Create a test session.
		let user_agent = "test-agent".to_string();
		let session = Session::new(
			*navigator.nutty_id(),
			user_agent.clone(),
			chrono::Duration::days(30),
		)
		.unwrap();

		// Act: Create the session.
		let created_session = repo
			.create_session(session.clone())
			.await
			.expect("Failed to create session");

		// Act: Delete the session.
		repo
			.delete_session(created_session.nutty_id())
			.await
			.expect("Failed to delete session");

		// Assert: The session no longer exists.
		let deleted_check = repo
			.get_session_by_id(created_session.nutty_id())
			.await
			.expect("Failed to check for deleted session");

		assert!(deleted_check.is_none(), "Session was not deleted");

		// Cleanup: Delete the test navigator.
		repo
			.delete_navigator(navigator.nutty_id())
			.await
			.expect("Failed to delete navigator");
	}
}
