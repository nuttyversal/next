use std::sync::Arc;

use super::models::PermissionCheck;
use super::models::PermissionResult;
use super::repository::AccessRepository;
use crate::models::NuttyId;

/// Service for managing access control operations.
#[derive(Clone)]
pub struct AccessService {
	repository: Arc<AccessRepository>,
}

impl AccessService {
	pub fn new(repository: AccessRepository) -> Self {
		Self {
			repository: Arc::new(repository),
		}
	}

	/// Check if a navigator has permission to perform an action.
	pub async fn can(&self, check: &PermissionCheck) -> Result<bool, AccessServiceError> {
		let result = self.repository.check_permission(check).await?;

		Ok(matches!(
			result,
			PermissionResult::GrantedGlobal
				| PermissionResult::GrantedResource
				| PermissionResult::GrantedOwnership
		))
	}

	/// Check permission and return detailed result.
	pub async fn check(
		&self,
		check: &PermissionCheck,
	) -> Result<PermissionResult, AccessServiceError> {
		self
			.repository
			.check_permission(check)
			.await
			.map_err(AccessServiceError::Repository)
	}

	/// Require permission - returns error if not granted.
	pub async fn require(&self, check: &PermissionCheck) -> Result<(), AccessServiceError> {
		let result = self.check(check).await?;

		match result {
			PermissionResult::GrantedGlobal
			| PermissionResult::GrantedResource
			| PermissionResult::GrantedOwnership => Ok(()),

			PermissionResult::Denied => Err(AccessServiceError::PermissionDenied {
				navigator_id: check.navigator_id().map(|id| id.to_string()),
				permission: check.permission().to_string(),
				resource: check
					.resource_type()
					.map(|t| format!("{}:{}", t, check.resource_id().unwrap())),
			}),
		}
	}

	/// Grant a global role to a navigator.
	pub async fn grant_global_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
	) -> Result<(), AccessServiceError> {
		self
			.repository
			.assign_global_role(navigator_id, role_name)
			.await
			.map_err(AccessServiceError::Repository)
	}

	/// Grant a resource role to a navigator.
	pub async fn grant_resource_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> Result<(), AccessServiceError> {
		self
			.repository
			.assign_resource_role(navigator_id, role_name, resource_type, resource_id)
			.await
			.map_err(AccessServiceError::Repository)
	}

	/// Revoke a global role from a navigator.
	pub async fn revoke_global_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
	) -> Result<(), AccessServiceError> {
		self
			.repository
			.remove_global_role(navigator_id, role_name)
			.await
			.map_err(AccessServiceError::Repository)
	}

	/// Revoke a resource role from a navigator.
	pub async fn revoke_resource_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> Result<(), AccessServiceError> {
		self
			.repository
			.remove_resource_role(navigator_id, role_name, resource_type, resource_id)
			.await
			.map_err(AccessServiceError::Repository)
	}

	/// Get all permissions for a navigator.
	pub async fn get_navigator_permissions(
		&self,
		navigator_id: &NuttyId,
	) -> Result<Vec<String>, AccessServiceError> {
		self
			.repository
			.get_navigator_permissions(navigator_id)
			.await
			.map_err(AccessServiceError::Repository)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum AccessServiceError {
	#[error("Repository error: {0}")]
	Repository(#[from] super::repository::AccessRepositoryError),

	#[error("Permission check error: {0}")]
	PermissionCheck(#[from] super::models::PermissionCheckError),

	#[error("Permission denied for navigator {navigator_id:?} on {permission} {resource:?}")]
	PermissionDenied {
		navigator_id: Option<String>,
		permission: String,
		resource: Option<String>,
	},
}

/// Extension trait for easy permission checking in other parts of the system.
pub trait AccessExt {
	/// Check if the navigator can perform an action.
	fn can(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
	) -> impl Future<Output = Result<bool, AccessServiceError>> + Send;

	/// Check if the navigator can perform an action on a specific resource.
	fn can_on(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> impl Future<Output = Result<bool, AccessServiceError>> + Send;

	/// Require permission for an action.
	fn require(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
	) -> impl Future<Output = Result<(), AccessServiceError>> + Send;

	/// Require permission for an action on a specific resource.
	fn require_on(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> impl Future<Output = Result<(), AccessServiceError>> + Send;
}

impl AccessExt for AccessService {
	async fn can(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
	) -> Result<bool, AccessServiceError> {
		let check = PermissionCheck::builder()
			.navigator(*navigator_id)
			.permission(permission.to_string())
			.try_build()
			.map_err(AccessServiceError::from)?;

		self.can(&check).await
	}

	async fn can_on(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> Result<bool, AccessServiceError> {
		let check = PermissionCheck::builder()
			.navigator(*navigator_id)
			.permission(permission.to_string())
			.resource(resource_type.to_string(), *resource_id)
			.try_build()
			.map_err(AccessServiceError::from)?;

		self.can(&check).await
	}

	async fn require(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
	) -> Result<(), AccessServiceError> {
		let check = PermissionCheck::builder()
			.navigator(*navigator_id)
			.permission(permission.to_string())
			.try_build()
			.map_err(AccessServiceError::from)?;

		self.require(&check).await
	}

	async fn require_on(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> Result<(), AccessServiceError> {
		let check = PermissionCheck::builder()
			.navigator(*navigator_id)
			.permission(permission.to_string())
			.resource(resource_type.to_string(), *resource_id)
			.try_build()
			.map_err(AccessServiceError::from)?;

		self.require(&check).await
	}
}

#[cfg(test)]
mod tests {
	use sqlx::PgPool;

	use super::*;
	use crate::access::repository::AccessRepository;
	use crate::models::NuttyId;

	/// Connect to the test database.
	async fn connect_to_test_database() -> PgPool {
		let database_url = std::env::var("DATABASE_URL").unwrap();

		PgPool::connect(&database_url)
			.await
			.expect("Failed to connect to test database")
	}

	/// Set up test data for access control tests.
	async fn setup_test_data(pool: &PgPool) -> (NuttyId, NuttyId, NuttyId, NuttyId) {
		// Create test navigators with unique names.
		let alice_id = NuttyId::now();
		let bob_id = NuttyId::now();
		let charlie_id = NuttyId::now();
		let resource_id = NuttyId::now();

		// Generate unique names using the Nutty ID.
		let alice_name = format!("alice_{}", alice_id.nid());
		let bob_name = format!("bob_{}", bob_id.nid());
		let charlie_name = format!("charlie_{}", charlie_id.nid());

		// Insert test navigators.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW()), ($4, $5, $6, 'test_pass', NOW(), NOW()), ($7, $8, $9, 'test_pass', NOW(), NOW())
			"#,
			alice_id.uuid(),
			alice_id.nid(),
			alice_name,
			bob_id.uuid(),
			bob_id.nid(),
			bob_name,
			charlie_id.uuid(),
			charlie_id.nid(),
			charlie_name,
		)
		.execute(pool)
		.await
		.expect("Failed to insert test navigators");

		// Insert test permissions.
		sqlx::query!(
			r#"
				INSERT INTO auth.permissions (name, description)
				VALUES 
					('content_blocks:read:all', 'Read all content blocks'),
					('content_blocks:write:all', 'Write all content blocks'),
					('content_blocks:write:own', 'Write own content blocks'),
					('content_blocks:read:resource', 'Read specific content block')
				ON CONFLICT (name) DO NOTHING
			"#
		)
		.execute(pool)
		.await
		.expect("Failed to insert test permissions");

		// Insert test roles.
		sqlx::query!(
			r#"
				INSERT INTO auth.roles (name, description)
				VALUES 
					('admin', 'Administrator role'),
					('editor', 'Editor role'),
					('viewer', 'Viewer role'),
					('block_owner', 'Content block owner role')
				ON CONFLICT (name) DO NOTHING
			"#
		)
		.execute(pool)
		.await
		.expect("Failed to insert test roles");

		// Insert role permissions.
		sqlx::query!(
			r#"
				INSERT INTO auth.role_permissions (role_name, permission_name)
				VALUES 
					('admin', 'content_blocks:read:all'),
					('admin', 'content_blocks:write:all'),
					('editor', 'content_blocks:read:all'),
					('viewer', 'content_blocks:read:resource'),
					('block_owner', 'content_blocks:write:own')
				ON CONFLICT (role_name, permission_name) DO NOTHING
			"#
		)
		.execute(pool)
		.await
		.expect("Failed to insert role permissions");

		(alice_id, bob_id, charlie_id, resource_id)
	}

	/// Clean up test data.
	async fn cleanup_test_data(pool: &PgPool, navigator_ids: &[NuttyId]) {
		// Clean up in reverse order of dependencies.
		for navigator_id in navigator_ids {
			sqlx::query!(
				r#"DELETE FROM auth.navigator_roles WHERE navigator_id = $1"#,
				navigator_id.uuid()
			)
			.execute(pool)
			.await
			.expect("Failed to cleanup navigator roles");

			sqlx::query!(
				r#"DELETE FROM auth.resource_roles WHERE navigator_id = $1"#,
				navigator_id.uuid()
			)
			.execute(pool)
			.await
			.expect("Failed to cleanup resource roles");

			sqlx::query!(
				r#"DELETE FROM auth.navigators WHERE id = $1"#,
				navigator_id.uuid()
			)
			.execute(pool)
			.await
			.expect("Failed to cleanup navigators");
		}
	}

	#[tokio::test]
	async fn test_can_with_global_permission() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign admin role to Alice.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Act & Assert: Check global permissions.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Bob has no roles, so should be denied.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(bob_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(!can_read);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_can_with_resource_permission() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Assign resource role to Alice.
		service
			.grant_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Act & Assert: Check resource-specific permissions.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:resource".to_string())
					.resource("content_block".to_string(), resource_id)
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Bob has no resource roles.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(bob_id)
					.permission("content_blocks:read:resource".to_string())
					.resource("content_block".to_string(), resource_id)
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(!can_read);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_check_returns_detailed_result() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign admin role to Alice.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Act & Assert: Check detailed permission results.
		let result = service
			.check(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedGlobal);

		// Bob has no roles, so should be denied.
		let result = service
			.check(
				&PermissionCheck::builder()
					.navigator(bob_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_require_succeeds_with_permission() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign admin role to Alice.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Act & Assert: Require permission should succeed.
		let result = service
			.require(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await;

		assert!(result.is_ok());

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_require_fails_without_permission() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Act & Assert: Require permission should fail for Bob who has no roles.
		let result = service
			.require(
				&PermissionCheck::builder()
					.navigator(bob_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await;

		assert!(result.is_err());
		match result {
			Err(AccessServiceError::PermissionDenied {
				navigator_id,
				permission,
				resource,
			}) => {
				assert_eq!(navigator_id, Some(bob_id.to_string()));
				assert_eq!(permission, "content_blocks:read:all");
				assert_eq!(resource, None);
			}
			_ => panic!("Expected PermissionDenied error"),
		}

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_require_fails_with_resource_permission() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Act & Assert: Require permission should fail for Bob who has no resource roles.
		let result = service
			.require(
				&PermissionCheck::builder()
					.navigator(bob_id)
					.permission("content_blocks:read:resource".to_string())
					.resource("content_block".to_string(), resource_id)
					.try_build()
					.unwrap(),
			)
			.await;

		assert!(result.is_err());

		match result {
			Err(AccessServiceError::PermissionDenied {
				navigator_id,
				permission,
				resource,
			}) => {
				assert_eq!(navigator_id, Some(bob_id.to_string()));
				assert_eq!(permission, "content_blocks:read:resource");
				assert_eq!(resource, Some(format!("content_block:{}", resource_id)));
			}

			_ => panic!("Expected PermissionDenied error"),
		}

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_grant_and_revoke_global_role() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Act: Grant global role.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to grant global role");

		// Assert: Verify the role was granted.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Act: Revoke global role.
		service
			.revoke_global_role(&alice_id, "admin")
			.await
			.expect("Failed to revoke global role");

		// Assert: Verify the role was revoked.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:all".to_string())
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(!can_read);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_grant_and_revoke_resource_role() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Act: Grant resource role.
		service
			.grant_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to grant resource role");

		// Assert: Verify the role was granted.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:resource".to_string())
					.resource("content_block".to_string(), resource_id)
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Act: Revoke resource role.
		service
			.revoke_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to revoke resource role");

		// Assert: Verify the role was revoked.
		let can_read = service
			.can(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("content_blocks:read:resource".to_string())
					.resource("content_block".to_string(), resource_id)
					.try_build()
					.unwrap(),
			)
			.await
			.expect("Failed to check permission");

		assert!(!can_read);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_get_navigator_permissions() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign multiple roles to Alice.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		service
			.grant_global_role(&alice_id, "editor")
			.await
			.expect("Failed to assign editor role");

		// Act: Get Alice's permissions.
		let permissions = service
			.get_navigator_permissions(&alice_id)
			.await
			.expect("Failed to get permissions");

		// Assert: Alice should have permissions from both roles.
		assert!(permissions.contains(&"content_blocks:read:all".to_string()));
		assert!(permissions.contains(&"content_blocks:write:all".to_string()));

		// Bob has no roles, so should have no permissions.
		let permissions = service
			.get_navigator_permissions(&bob_id)
			.await
			.expect("Failed to get permissions");

		assert!(permissions.is_empty());

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_access_ext_can() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign admin role to Alice.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Act & Assert: Test AccessExt::can method.
		let can_read = AccessExt::can(&service, &alice_id, "content_blocks:read:all")
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Bob has no roles.
		let can_read = AccessExt::can(&service, &bob_id, "content_blocks:read:all")
			.await
			.expect("Failed to check permission");

		assert!(!can_read);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_access_ext_can_on() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Assign resource role to Alice.
		service
			.grant_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Act & Assert: Test AccessExt::can_on method.
		let can_read = service
			.can_on(
				&alice_id,
				"content_blocks:read:resource",
				"content_block",
				&resource_id,
			)
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Bob has no resource roles.
		let can_read = service
			.can_on(
				&bob_id,
				"content_blocks:read:resource",
				"content_block",
				&resource_id,
			)
			.await
			.expect("Failed to check permission");

		assert!(!can_read);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_access_ext_require() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign admin role to Alice.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Act & Assert: Test AccessExt::require method.
		let result = AccessExt::require(&service, &alice_id, "content_blocks:read:all").await;
		assert!(result.is_ok());

		// Bob has no roles.
		let result = AccessExt::require(&service, &bob_id, "content_blocks:read:all").await;
		assert!(result.is_err());

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_access_ext_require_on() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Assign resource role to Alice.
		service
			.grant_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Act & Assert: Test AccessExt::require_on method.
		let result = service
			.require_on(
				&alice_id,
				"content_blocks:read:resource",
				"content_block",
				&resource_id,
			)
			.await;

		assert!(result.is_ok());

		// Bob has no resource roles.
		let result = service
			.require_on(
				&bob_id,
				"content_blocks:read:resource",
				"content_block",
				&resource_id,
			)
			.await;

		assert!(result.is_err());

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_permission_hierarchy() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Alice has global admin role.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Bob has resource-specific role.
		service
			.grant_resource_role(&bob_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Charlie has no roles.

		// Act & Assert: Test permission hierarchy.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		// Alice should get global permission.
		let result = service
			.check(&check)
			.await
			.expect("Failed to check permission");
		assert_eq!(result, PermissionResult::GrantedGlobal);

		let check = PermissionCheck::builder()
			.navigator(bob_id)
			.permission("content_blocks:read:resource".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		// Bob should get resource permission.
		let result = service
			.check(&check)
			.await
			.expect("Failed to check permission");
		assert_eq!(result, PermissionResult::GrantedResource);

		let check = PermissionCheck::builder()
			.navigator(charlie_id)
			.permission("content_blocks:read:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		// Charlie should be denied (no permissions).
		let result = service
			.check(&check)
			.await
			.expect("Failed to check permission");
		assert_eq!(result, PermissionResult::Denied);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_error_handling() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Test with non-existent role.
		let result = service
			.grant_global_role(&alice_id, "non_existent_role")
			.await;

		assert!(result.is_err());

		// Test with non-existent permission.
		let result = service
			.can(
				&PermissionCheck::builder()
					.navigator(alice_id)
					.permission("non_existent:permission".to_string())
					.try_build()
					.unwrap(),
			)
			.await;

		assert!(result.is_ok()); // Should return false, not error
		assert!(!result.unwrap());

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_idempotent_role_operations() {
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let service = AccessService::new(repo);
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Act: Grant the same global role twice.
		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to grant global role");

		service
			.grant_global_role(&alice_id, "admin")
			.await
			.expect("Failed to grant global role again");

		// Assert: Should still have the permission.
		let can_read = AccessExt::can(&service, &alice_id, "content_blocks:read:all")
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Act: Grant the same resource role twice.
		service
			.grant_resource_role(&bob_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to grant resource role");

		service
			.grant_resource_role(&bob_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to grant resource role again");

		// Assert: Should still have the permission.
		let can_read = service
			.can_on(
				&bob_id,
				"content_blocks:read:resource",
				"content_block",
				&resource_id,
			)
			.await
			.expect("Failed to check permission");

		assert!(can_read);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}
}
