use sqlx::PgPool;
use thiserror::Error;

use crate::access::models::PermissionCheck;
use crate::access::models::PermissionResult;
use crate::access::models::ResourceRole;
use crate::models::NuttyId;

/// Repository for managing access control data.
#[derive(Clone)]
pub struct AccessRepository {
	/// The PostgreSQL database pool.
	pool: PgPool,
}

impl AccessRepository {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}

	/// Check if a navigator has a specific permission through the three-tier system:
	///
	/// - Global roles (e.g., "content_blocks:write:all").
	/// - Resource-specific roles (e.g., "content_blocks:write" on #123).
	/// - Ownership permissions (e.g., "content_blocks:write:own" if navigator owns #123).
	pub async fn check_permission(
		&self,
		check: &PermissionCheck,
	) -> Result<PermissionResult, AccessRepositoryError> {
		let navigator_id = match check.navigator_id() {
			Some(id) => id,
			None => return Ok(PermissionResult::Denied),
		};

		// Tier 1: Check global roles.
		if self
			.has_global_permission(navigator_id, check.permission())
			.await?
		{
			return Ok(PermissionResult::GrantedGlobal);
		}

		// Tier 2: Check resource-specific roles.
		if let (Some(resource_type), Some(resource_id)) = (check.resource_type(), check.resource_id())
		{
			if self
				.has_resource_permission(navigator_id, check.permission(), resource_type, resource_id)
				.await?
			{
				return Ok(PermissionResult::GrantedResource);
			}
		}

		// Tier 3: Check ownership permissions.
		if let (Some(resource_type), Some(resource_id)) = (check.resource_type(), check.resource_id())
		{
			if self
				.is_owner(navigator_id, resource_type, resource_id)
				.await? && self
				.has_ownership_permission(navigator_id, check.permission())
				.await?
			{
				return Ok(PermissionResult::GrantedOwnership);
			}
		}

		Ok(PermissionResult::Denied)
	}

	/// Check if a navigator has a permission through global roles.
	async fn has_global_permission(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
	) -> Result<bool, AccessRepositoryError> {
		let result = sqlx::query!(
			r#"
				SELECT EXISTS(
					SELECT 1 FROM auth.navigator_roles nr
					JOIN auth.role_permissions rp ON nr.role_name = rp.role_name
					WHERE nr.navigator_id = $1 AND rp.permission_name = $2
				) as "exists!"
			"#,
			navigator_id.uuid(),
			permission
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(result.exists)
	}

	/// Check if a navigator has a permission through resource-specific roles.
	async fn has_resource_permission(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> Result<bool, AccessRepositoryError> {
		let result = sqlx::query!(
			r#"
				SELECT EXISTS(
					SELECT 1 FROM auth.resource_roles rr
					JOIN auth.role_permissions rp ON rr.role_name = rp.role_name
					WHERE rr.navigator_id = $1
						AND rp.permission_name = $2
						AND rr.resource_type = $3
						AND rr.resource_id = $4
				) as "exists!"
			"#,
			navigator_id.uuid(),
			permission,
			resource_type,
			resource_id.uuid()
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(result.exists)
	}

	/// Check if a navigator owns a resource.
	async fn is_owner(
		&self,
		_navigator_id: &NuttyId,
		_resource_type: &str,
		_resource_id: &NuttyId,
	) -> Result<bool, AccessRepositoryError> {
		// TODO: Implement the is_owner method.
		Ok(false)
	}

	/// Check if a navigator has ownership permission.
	async fn has_ownership_permission(
		&self,
		navigator_id: &NuttyId,
		permission: &str,
	) -> Result<bool, AccessRepositoryError> {
		if permission.ends_with(":own") {
			self.has_global_permission(navigator_id, permission).await
		} else {
			Ok(false)
		}
	}

	/// Get all permissions for a navigator.
	pub async fn get_navigator_permissions(
		&self,
		navigator_id: &NuttyId,
	) -> Result<Vec<String>, AccessRepositoryError> {
		let rows = sqlx::query!(
			r#"
				SELECT DISTINCT rp.permission_name
				FROM auth.navigator_roles nr
				JOIN auth.role_permissions rp ON nr.role_name = rp.role_name
				WHERE nr.navigator_id = $1
			"#,
			navigator_id.uuid()
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(rows.into_iter().map(|row| row.permission_name).collect())
	}

	/// Get all resource roles for a navigator.
	pub async fn get_navigator_resource_roles(
		&self,
		navigator_id: &NuttyId,
	) -> Result<Vec<ResourceRole>, AccessRepositoryError> {
		let rows = sqlx::query_as(
			r#"
				SELECT id, navigator_id, role_name, resource_type, resource_id, created_at, updated_at
				FROM auth.resource_roles
				WHERE navigator_id = $1
			"#,
		)
		.bind(navigator_id.uuid())
		.fetch_all(&self.pool)
		.await?;

		Ok(rows)
	}

	/// Assign a global role to a navigator.
	pub async fn assign_global_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
	) -> Result<(), AccessRepositoryError> {
		let nutty_id = NuttyId::now();

		sqlx::query!(
			r#"
				INSERT INTO auth.navigator_roles (id, nutty_id, navigator_id, role_name)
				VALUES ($1, $2, $3, $4)
				ON CONFLICT (navigator_id, role_name) DO NOTHING
			"#,
			nutty_id.uuid(),
			nutty_id.nid(),
			navigator_id.uuid(),
			role_name
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	/// Assign a resource role to a navigator.
	pub async fn assign_resource_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> Result<(), AccessRepositoryError> {
		let nutty_id = NuttyId::now();

		sqlx::query!(
			r#"
				INSERT INTO auth.resource_roles (id, nutty_id, navigator_id, role_name, resource_type, resource_id)
				VALUES ($1, $2, $3, $4, $5, $6)
			"#,
			nutty_id.uuid(),
			nutty_id.nid(),
			navigator_id.uuid(),
			role_name,
			resource_type,
			resource_id.uuid()
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	/// Remove a global role from a navigator.
	pub async fn remove_global_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
	) -> Result<(), AccessRepositoryError> {
		sqlx::query!(
			r#"
				DELETE FROM auth.navigator_roles
				WHERE navigator_id = $1 AND role_name = $2
			"#,
			navigator_id.uuid(),
			role_name
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	/// Remove a resource role from a navigator.
	pub async fn remove_resource_role(
		&self,
		navigator_id: &NuttyId,
		role_name: &str,
		resource_type: &str,
		resource_id: &NuttyId,
	) -> Result<(), AccessRepositoryError> {
		sqlx::query!(
			r#"
				DELETE FROM auth.resource_roles
				WHERE navigator_id = $1
					AND role_name = $2
					AND resource_type = $3
					AND resource_id = $4
			"#,
			navigator_id.uuid(),
			role_name,
			resource_type,
			resource_id.uuid()
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum AccessRepositoryError {
	#[error("Database error: {0}")]
	Database(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
	use sqlx::PgPool;

	use super::*;
	use crate::access::models::PermissionCheck;
	use crate::access::models::PermissionResult;

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
				VALUES
					($1, $2, $3, 'hashed_password', NOW(), NOW()),
					($4, $5, $6, 'hashed_password', NOW(), NOW()),
					($7, $8, $9, 'hashed_password', NOW(), NOW())
			"#,
			alice_id.uuid(),
			alice_id.nid(),
			alice_name,
			bob_id.uuid(),
			bob_id.nid(),
			bob_name,
			charlie_id.uuid(),
			charlie_id.nid(),
			charlie_name
		)
		.execute(pool)
		.await
		.expect("Failed to insert test navigators");

		// Insert test permissions.
		sqlx::query!(
			r#"
				INSERT INTO auth.permissions (name, description)
				VALUES
					('content_blocks:read:all', 'Can read all content blocks'),
					('content_blocks:write:all', 'Can write all content blocks'),
					('content_blocks:read:own', 'Can read own content blocks'),
					('content_blocks:write:own', 'Can write own content blocks'),
					('content_blocks:read', 'Can read specific content block'),
					('content_blocks:write', 'Can write specific content block')
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
					('block_owner', 'Block owner role')
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
					('editor', 'content_blocks:write:own'),
					('viewer', 'content_blocks:read:all'),
					('block_owner', 'content_blocks:read:own'),
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
				r#"
					DELETE FROM auth.resource_roles WHERE navigator_id = $1
				"#,
				navigator_id.uuid()
			)
			.execute(pool)
			.await
			.expect("Failed to cleanup resource roles");

			sqlx::query!(
				r#"
					DELETE FROM auth.navigator_roles WHERE navigator_id = $1
				"#,
				navigator_id.uuid()
			)
			.execute(pool)
			.await
			.expect("Failed to cleanup navigator roles");
		}

		sqlx::query!(
			r#"
				DELETE FROM auth.navigators WHERE id IN (
					SELECT unnest($1::uuid[])
				)
			"#,
			&navigator_ids
				.iter()
				.map(|id| *id.uuid())
				.collect::<Vec<_>>()
		)
		.execute(pool)
		.await
		.expect("Failed to cleanup navigators");
	}

	#[tokio::test]
	async fn test_check_permission_global_role() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign admin role to Alice.
		repo
			.assign_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Act & Assert: Check global permissions.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedGlobal);

		// Bob has no roles. Should be denied.
		let check = PermissionCheck::builder()
			.navigator(bob_id)
			.permission("content_blocks:read:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_check_permission_resource_role() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Assign resource role to Alice.
		repo
			.assign_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Act & Assert: Check resource-specific permissions.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedResource);

		// Bob has no resource role, so should be denied
		let check = PermissionCheck::builder()
			.navigator(bob_id)
			.permission("content_blocks:read:all".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);

		// Cleanup
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_check_permission_ownership() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Assign ownership permission to Alice.
		repo
			.assign_global_role(&alice_id, "block_owner")
			.await
			.expect("Failed to assign block_owner role");

		// Act & Assert: Check ownership permissions.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:write:own".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		// Alice has the `block_owner` role which gives her `content_blocks:write:own`
		// permission globally. Since `is_owner` currently returns false, she gets
		// `GrantedGlobal` instead of `GrantedOwnership.`
		//
		// TODO: Update this assertion after implementing the is_owner method.
		assert_eq!(result, PermissionResult::GrantedGlobal);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_check_permission_no_navigator() {
		// Arrange: Set up repository.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool);

		// Act & Assert: Check permission without navigator (anonymous).
		let check = PermissionCheck::builder()
			.permission("content_blocks:read:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);
	}

	#[tokio::test]
	async fn test_get_navigator_permissions() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign multiple roles to Alice.
		repo
			.assign_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		repo
			.assign_global_role(&alice_id, "editor")
			.await
			.expect("Failed to assign editor role");

		// Act: Get Alice's permissions.
		let permissions = repo
			.get_navigator_permissions(&alice_id)
			.await
			.expect("Failed to get permissions");

		// Assert: Alice should have permissions from both roles.
		assert!(permissions.contains(&"content_blocks:read:all".to_string()));
		assert!(permissions.contains(&"content_blocks:write:all".to_string()));
		assert!(permissions.contains(&"content_blocks:write:own".to_string()));

		// Bob has no roles, so should have no permissions.
		let permissions = repo
			.get_navigator_permissions(&bob_id)
			.await
			.expect("Failed to get permissions");

		assert!(permissions.is_empty());

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_get_navigator_resource_roles() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Assign resource roles to Alice.
		repo
			.assign_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		let resource_id_2 = NuttyId::now();

		repo
			.assign_resource_role(&alice_id, "editor", "content_block", &resource_id_2)
			.await
			.expect("Failed to assign second resource role");

		// Act: Get Alice's resource roles.
		let resource_roles = repo
			.get_navigator_resource_roles(&alice_id)
			.await
			.expect("Failed to get resource roles");

		// Assert: Alice should have both resource roles.
		assert_eq!(resource_roles.len(), 2);

		assert!(
			resource_roles
				.iter()
				.any(|rr| rr.role_name() == "viewer" && rr.resource_id() == &resource_id)
		);

		assert!(
			resource_roles
				.iter()
				.any(|rr| rr.role_name() == "editor" && rr.resource_id() == &resource_id_2)
		);

		// Bob has no resource roles.
		let resource_roles = repo
			.get_navigator_resource_roles(&bob_id)
			.await
			.expect("Failed to get resource roles");

		assert!(resource_roles.is_empty());

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_assign_global_role() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Act: Assign global role.
		repo
			.assign_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign global role");

		// Assert: Verify the role was assigned.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedGlobal);

		// Test idempotency. Assigning the same role again should not fail.
		repo
			.assign_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign global role again");

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_assign_resource_role() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Act: Assign resource role.
		repo
			.assign_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Assert: Verify the role was assigned.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedResource);

		// Test idempotency. Assigning the same role again should not fail.
		repo
			.assign_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role again");

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_remove_global_role() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Assign global role.
		repo
			.assign_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign global role");

		// Verify role is assigned.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedGlobal);

		// Act: Remove global role.
		repo
			.remove_global_role(&alice_id, "admin")
			.await
			.expect("Failed to remove global role");

		// Assert: Verify the role was removed.
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");
		assert_eq!(result, PermissionResult::Denied);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_remove_resource_role() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Assign resource role.
		repo
			.assign_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Verify role is assigned.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedResource);

		// Act: Remove resource role.
		repo
			.remove_resource_role(&alice_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to remove resource role");

		// Assert: Verify the role was removed.
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_permission_check_builder() {
		let navigator_id = NuttyId::now();
		let resource_id = NuttyId::now();

		let check = PermissionCheck::builder()
			.navigator(navigator_id)
			.permission("test:permission".to_string())
			.resource("test_resource".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		assert_eq!(check.navigator_id(), Some(&navigator_id));
		assert_eq!(check.permission(), "test:permission");
		assert_eq!(check.resource_type(), Some("test_resource"));
		assert_eq!(check.resource_id(), Some(&resource_id));

		// Test build without permission (should fail).
		let result = PermissionCheck::builder()
			.navigator(navigator_id)
			.try_build();

		assert!(result.is_err());
	}

	#[tokio::test]
	async fn test_permission_hierarchy() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, resource_id) = setup_test_data(&pool).await;

		// Alice has global admin role.
		repo
			.assign_global_role(&alice_id, "admin")
			.await
			.expect("Failed to assign admin role");

		// Bob has resource-specific role.
		repo
			.assign_resource_role(&bob_id, "viewer", "content_block", &resource_id)
			.await
			.expect("Failed to assign resource role");

		// Charlie has no roles.
		// …

		// Act & Assert: Test permission hierarchy.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:read:all".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		// Alice should get global permission.
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedGlobal);

		let check = PermissionCheck::builder()
			.navigator(bob_id)
			.permission("content_blocks:read:all".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		// Bob should get resource permission.
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedResource);

		let check = PermissionCheck::builder()
			.navigator(charlie_id)
			.permission("content_blocks:read:all".to_string())
			.resource("content_block".to_string(), resource_id)
			.try_build()
			.expect("Failed to build permission check");

		// Charlie should be denied (no permissions).
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}

	#[tokio::test]
	async fn test_ownership_permission_logic() {
		// Arrange: Set up test data.
		let pool = connect_to_test_database().await;
		let repo = AccessRepository::new(pool.clone());
		let (alice_id, bob_id, charlie_id, _) = setup_test_data(&pool).await;

		// Alice has ownership permission.
		repo
			.assign_global_role(&alice_id, "block_owner")
			.await
			.expect("Failed to assign block_owner role");

		// Bob has no ownership permission.
		// …

		// Act & Assert: Test ownership permission logic.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:write:own".to_string())
			.try_build()
			.expect("Failed to build permission check");

		// Alice should have ownership permission.
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::GrantedGlobal);

		let check = PermissionCheck::builder()
			.navigator(bob_id)
			.permission("content_blocks:write:own".to_string())
			.try_build()
			.expect("Failed to build permission check");

		// Bob should not have ownership permission.
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);

		// Test non-ownership permission.
		let check = PermissionCheck::builder()
			.navigator(alice_id)
			.permission("content_blocks:write:all".to_string())
			.try_build()
			.expect("Failed to build permission check");

		// Should be denied because it's not an ownership permission.
		let result = repo
			.check_permission(&check)
			.await
			.expect("Failed to check permission");

		assert_eq!(result, PermissionResult::Denied);

		// Cleanup.
		cleanup_test_data(&pool, &[alice_id, bob_id, charlie_id]).await;
	}
}
