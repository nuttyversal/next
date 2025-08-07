use crate::access::service::AccessService;
use crate::content::repository::ContentRepository;
use crate::content::repository::ContentRepositoryError;
use crate::models::ContentBlock;
use crate::models::ContentContext;
use crate::models::ContentLink;
use crate::models::DissociatedNuttyId;
use crate::utilities::repository::Repository;
use crate::utilities::repository::TransactionExt;

#[derive(Clone)]
pub struct ContentService {
	/// The content repository to use for storing and retrieving content.
	repository: ContentRepository,

	/// The access service to use for permission checking.
	access_service: AccessService,
}

impl ContentService {
	/// Create a new content service with the given repository and access service.
	pub fn new(repository: ContentRepository, access_service: AccessService) -> Self {
		ContentService {
			repository,
			access_service,
		}
	}

	/// Get a content block's context.
	pub async fn get_content_block_context(
		&self,
		nutty_id: &DissociatedNuttyId,
	) -> Result<ContentContext, ContentServiceError> {
		// Get the content block.
		let content_block = self
			.repository
			.get_content_block(nutty_id)
			.await
			.map_err(ContentServiceError::FetchContentBlock)?
			.ok_or(ContentServiceError::ContentBlockNotFound)?;

		// Get the ancestor blocks.
		let ancestors = self
			.repository
			.get_ancestor_blocks(nutty_id)
			.await
			.map_err(ContentServiceError::FetchAncestorBlocks)?;

		// Get the descendant blocks.
		let descendants = self
			.repository
			.get_descendant_blocks(nutty_id)
			.await
			.map_err(ContentServiceError::FetchDescendantBlocks)?;

		// Get immediate children.
		let children_ids = descendants
			.iter()
			.filter(|block| (block.parent_id.map(|i| i.nid()) == Some(nutty_id.nid())))
			.map(|block| *block.nutty_id())
			.collect::<Vec<_>>();

		// Get outbound links (references).
		let outbound_links = self
			.repository
			.get_content_links_from(content_block.nutty_id())
			.await
			.map_err(ContentServiceError::FetchOutboundLinks)?;

		// Get inbound links (backlinks).
		let inbound_links = self
			.repository
			.get_content_links_to(content_block.nutty_id())
			.await
			.map_err(ContentServiceError::FetchInboundLinks)?;

		// Build the block cache.
		let mut block_cache = std::collections::HashMap::new();

		// Add the main content block to the cache.
		block_cache.insert(*content_block.nutty_id(), content_block.clone());

		// Add ancestor blocks to the cache.
		for block in &ancestors {
			block_cache.insert(*block.nutty_id(), block.clone());
		}

		// Add descendant blocks to the cache.
		for block in &descendants {
			block_cache.insert(*block.nutty_id(), block.clone());
		}

		// Extract reference and backlink IDs.
		let reference_ids = outbound_links.iter().map(|link| link.target_id).collect();
		let backlink_ids = inbound_links.iter().map(|link| link.source_id).collect();

		// Create the content context.
		let context = ContentContext::builder()
			.block_id(*content_block.nutty_id())
			.parent_id(content_block.parent_id)
			.children_ids(children_ids)
			.reference_ids(reference_ids)
			.backlink_ids(backlink_ids)
			.block_cache(block_cache)
			.try_build()
			.map_err(|err| ContentServiceError::BuildContentContext(err.to_string()))?;

		Ok(context)
	}

	/// Save a content block.
	pub async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ContentServiceError> {
		self
			.repository
			.with_transaction(|tx| {
				Box::pin(async move {
					// Save the content block.
					let content_block = self
						.repository
						.upsert_content_block_tx(tx.as_executor(), content_block.clone())
						.await
						.map_err(ContentServiceError::SaveContentBlock)?;

					// Parse tags from the content block.
					let target_tags = content_block.content.parse_target_tags();

					// Resolve [NuttyTag] references.
					let target_ids = self
						.repository
						.resolve_nutty_ids_tx(
							tx.as_executor(),
							target_tags
								.iter()
								.map(|tag| tag.nutty_id())
								.collect::<Vec<_>>(),
						)
						.await;

					// Delete orphaned content links.
					self
						.repository
						.delete_orphaned_content_links_tx(
							tx.as_executor(),
							content_block.nutty_id(),
							&target_ids,
						)
						.await
						.map_err(ContentServiceError::DeleteContentLinks)?;

					// Create new content links.
					let content_links: Vec<ContentLink> = target_ids
						.iter()
						.map(|target_id| ContentLink::now(*content_block.nutty_id(), *target_id))
						.collect();

					// Save the content links.
					self
						.repository
						.upsert_content_links_tx(tx.as_executor(), &content_links)
						.await
						.map_err(ContentServiceError::SaveContentLink)?;

					// Return the saved content block.
					Ok(content_block)
				})
			})
			.await
	}

	/// Check if a navigator has access to a content block or any of its ancestors.
	pub async fn check_content_block_access(
		&self,
		navigator_id: &crate::models::NuttyId,
		block_id: &DissociatedNuttyId,
	) -> Result<bool, ContentServiceError> {
		// First, resolve the DissociatedNuttyId to a NuttyId.
		let resolved_block_id = self
			.repository
			.resolve_nutty_id(block_id.clone())
			.await
			.map_err(ContentServiceError::FetchContentBlock)?;

		// 1. Check if the navigator has global read permission.
		let can_access_globally = self
			.access_service
			.can_permission(navigator_id, "content_blocks:read:all")
			.await
			.map_err(ContentServiceError::AccessControl)?;

		if can_access_globally {
			return Ok(true);
		}

		// 2. Check if the navigator has access to the requested block.
		let can_access_block = self
			.access_service
			.can_on_resource(
				navigator_id,
				"content_blocks:read:resource",
				"content_block",
				&resolved_block_id,
			)
			.await
			.map_err(ContentServiceError::AccessControl)?;

		if can_access_block {
			return Ok(true);
		}

		// 3. Check if the navigator has ownership permission.
		let can_access_own = self
			.access_service
			.can_permission(navigator_id, "content_blocks:read:own")
			.await
			.map_err(ContentServiceError::AccessControl)?;

		if can_access_own {
			// Check if the navigator owns the block.
			let content_block = self
				.repository
				.get_content_block(block_id)
				.await
				.map_err(ContentServiceError::FetchContentBlock)?
				.ok_or(ContentServiceError::ContentBlockNotFound)?;

			if let Some(owner_id) = content_block.owner_id {
				if owner_id == *navigator_id {
					return Ok(true);
				}
			}
		}

		// 4. Check if the navigator has access to any ancestor blocks.
		let ancestors = self
			.repository
			.get_ancestor_blocks(block_id)
			.await
			.map_err(ContentServiceError::FetchAncestorBlocks)?;

		for ancestor in &ancestors {
			let can_access_ancestor = self
				.access_service
				.can_on_resource(
					navigator_id,
					"content_blocks:read:resource",
					"content_block",
					ancestor.nutty_id(),
				)
				.await
				.map_err(ContentServiceError::AccessControl)?;

			if can_access_ancestor {
				return Ok(true);
			}
		}

		Ok(false)
	}

	/// Check if a navigator has write access to a content block or any of its ancestors.
	pub async fn check_content_block_write_access(
		&self,
		navigator_id: &crate::models::NuttyId,
		block_id: &DissociatedNuttyId,
	) -> Result<bool, ContentServiceError> {
		// First, resolve the DissociatedNuttyId to a NuttyId.
		let resolved_block_id = self
			.repository
			.resolve_nutty_id(block_id.clone())
			.await
			.map_err(ContentServiceError::FetchContentBlock)?;

		// 1. Check if the navigator has global write permission.
		let can_write_globally = self
			.access_service
			.can_permission(navigator_id, "content_blocks:write:all")
			.await
			.map_err(ContentServiceError::AccessControl)?;

		if can_write_globally {
			return Ok(true);
		}

		// 2. Check if the navigator has direct write access to the requested block.
		let can_write_block = self
			.access_service
			.can_on_resource(
				navigator_id,
				"content_blocks:write",
				"content_block",
				&resolved_block_id,
			)
			.await
			.map_err(ContentServiceError::AccessControl)?;

		if can_write_block {
			return Ok(true);
		}

		// 3. Check if the navigator has ownership write permission.
		let can_write_own = self
			.access_service
			.can_permission(navigator_id, "content_blocks:write:own")
			.await
			.map_err(ContentServiceError::AccessControl)?;

		if can_write_own {
			// Check if the navigator owns the block.
			let content_block = self
				.repository
				.get_content_block(block_id)
				.await
				.map_err(ContentServiceError::FetchContentBlock)?
				.ok_or(ContentServiceError::ContentBlockNotFound)?;

			if let Some(owner_id) = content_block.owner_id {
				if owner_id == *navigator_id {
					return Ok(true);
				}
			}
		}

		// 4. Check if the navigator has write access to any ancestor blocks.
		let ancestors = self
			.repository
			.get_ancestor_blocks(block_id)
			.await
			.map_err(ContentServiceError::FetchAncestorBlocks)?;

		for ancestor in &ancestors {
			let can_write_ancestor = self
				.access_service
				.can_on_resource(
					navigator_id,
					"content_blocks:write",
					"content_block",
					ancestor.nutty_id(),
				)
				.await
				.map_err(ContentServiceError::AccessControl)?;

			if can_write_ancestor {
				return Ok(true);
			}
		}

		Ok(false)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ContentServiceError {
	#[error("Failed to save content block: {0}")]
	SaveContentBlock(#[source] ContentRepositoryError),

	#[error("Failed to save content link: {0}")]
	SaveContentLink(#[source] ContentRepositoryError),

	#[error("Failed to delete content links: {0}")]
	DeleteContentLinks(#[source] ContentRepositoryError),

	#[error("Database error: {0}")]
	Database(#[from] sqlx::Error),

	#[error("Content block not found")]
	ContentBlockNotFound,

	#[error("Failed to fetch content block: {0}")]
	FetchContentBlock(#[source] ContentRepositoryError),

	#[error("Failed to fetch ancestor blocks: {0}")]
	FetchAncestorBlocks(#[source] ContentRepositoryError),

	#[error("Failed to fetch descendant blocks: {0}")]
	FetchDescendantBlocks(#[source] ContentRepositoryError),

	#[error("Failed to fetch outbound links: {0}")]
	FetchOutboundLinks(#[source] ContentRepositoryError),

	#[error("Failed to fetch inbound links: {0}")]
	FetchInboundLinks(#[source] ContentRepositoryError),

	#[error("Failed to build content context: {0}")]
	BuildContentContext(String),

	#[error("Access control error: {0}")]
	AccessControl(#[source] crate::access::service::AccessServiceError),
}

#[cfg(test)]
mod tests {
	use sqlx::Pool;
	use sqlx::Postgres;
	use sqlx::postgres::PgPoolOptions;

	use super::*;
	use crate::access::repository::AccessRepository;
	use crate::access::service::AccessService;
	use crate::content::repository::ContentRepository;
	use crate::models::BlockContent;
	use crate::models::ContentBlock;
	use crate::models::FractionalIndex;
	use crate::models::NuttyId;

	async fn connect_to_test_database() -> Pool<Postgres> {
		let database_url = std::env::var("DATABASE_URL").unwrap();

		PgPoolOptions::new()
			.max_connections(5)
			.connect(&database_url)
			.await
			.expect("Failed to connect to test database")
	}

	#[tokio::test]
	async fn test_get_content_block_context() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Arrange: Create a hierarchy of content blocks.
		let parent_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Parent Page".to_string(),
			},
		);

		let middle_block = ContentBlock::now(
			Some(*parent_block.nutty_id()),
			FractionalIndex::between(&FractionalIndex::start(), &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Middle Page".to_string(),
			},
		);

		let child_block = ContentBlock::now(
			Some(*middle_block.nutty_id()),
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Child Page".to_string(),
			},
		);

		let sibling_block = ContentBlock::now(
			Some(*middle_block.nutty_id()),
			FractionalIndex::between(&child_block.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Sibling Page".to_string(),
			},
		);

		// Arrange: Create a reference block.
		let reference_block = ContentBlock::now(
			None,
			FractionalIndex::end(),
			BlockContent::Page {
				title: "Reference Page".to_string(),
			},
		);

		// Act: Save all the blocks.
		service
			.repository
			.upsert_content_block(parent_block.clone())
			.await
			.expect("Failed to save parent block");

		service
			.repository
			.upsert_content_block(middle_block.clone())
			.await
			.expect("Failed to save middle block");

		service
			.repository
			.upsert_content_block(child_block.clone())
			.await
			.expect("Failed to save child block");

		service
			.repository
			.upsert_content_block(sibling_block.clone())
			.await
			.expect("Failed to save sibling block");

		service
			.repository
			.upsert_content_block(reference_block.clone())
			.await
			.expect("Failed to save reference block");

		// Arrange: Create content links.
		let link = ContentLink::now(*middle_block.nutty_id(), *reference_block.nutty_id());
		service
			.repository
			.upsert_content_link(link)
			.await
			.expect("Failed to create link");

		// Act: Get the context for the middle block.
		let context = service
			.get_content_block_context(&middle_block.nutty_id().into())
			.await
			.expect("Failed to get content context");

		// Assert: The context has the correct block ID.
		assert_eq!(context.block_id(), middle_block.nutty_id());

		// Assert: The context has the correct parent ID.
		assert_eq!(context.parent_id(), Some(parent_block.nutty_id()));

		// Assert: The context has the correct children IDs.
		let children_ids = context.children_ids();
		assert_eq!(children_ids.len(), 2);
		assert!(children_ids.contains(child_block.nutty_id()));
		assert!(children_ids.contains(sibling_block.nutty_id()));

		// Assert: The context has the correct reference IDs.
		let reference_ids = context.reference_ids();
		assert_eq!(reference_ids.len(), 1);
		assert_eq!(reference_ids[0], *reference_block.nutty_id());

		// Assert: The cache contains all the expected blocks.
		let cache = context.block_cache();
		assert!(cache.contains_key(parent_block.nutty_id()));
		assert!(cache.contains_key(middle_block.nutty_id()));
		assert!(cache.contains_key(child_block.nutty_id()));
		assert!(cache.contains_key(sibling_block.nutty_id()));

		// References may or may not be included automatically in the cache
		// depending on implementation details.

		// Assert: The context has the correct children IDs.
		let children_ids = context.children_ids();
		assert_eq!(children_ids.len(), 2);
		assert!(children_ids.contains(child_block.nutty_id()));
		assert!(children_ids.contains(sibling_block.nutty_id()));

		// Get context for a child block to test different parent/children relationships.
		let child_context = service
			.get_content_block_context(&child_block.nutty_id().into())
			.await
			.expect("Failed to get child content context");

		// Assert: The context has the correct parent ID.
		assert_eq!(child_context.parent_id(), Some(middle_block.nutty_id()));

		// Assert: The context has no children.
		assert_eq!(child_context.children_ids().len(), 0);

		// Assert: The cache still contains ancestors and descendants.
		let child_cache = child_context.block_cache();
		assert!(child_cache.contains_key(parent_block.nutty_id()));
		assert!(child_cache.contains_key(middle_block.nutty_id()));
		assert!(child_cache.contains_key(child_block.nutty_id()));
	}

	#[tokio::test]
	async fn test_save_content_block() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block = ContentBlock::now(
			None,
			FractionalIndex::between(&source_block.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Target Page".to_string(),
			},
		);

		// Act: Save the target block first.
		let saved_target = service
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a source block with a link to the target.
		let source_with_link = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Paragraph {
				markdown: format!("This links to [[{}]]", saved_target.nutty_id().nid()),
			},
		);

		// Act: Save the source block.
		let saved_source = service
			.save_content_block(source_with_link.clone())
			.await
			.expect("Failed to save source block");

		// Assert: The source block was saved correctly.
		assert_eq!(saved_source.nutty_id(), source_with_link.nutty_id());
		assert_eq!(saved_source.parent_id, source_with_link.parent_id);
		assert!(matches!(
			&saved_source.content,
			BlockContent::Paragraph { markdown } if markdown == &format!("This links to [[{}]]", saved_target.nutty_id().nid())
		));

		// Assert: A link was created between the blocks.
		let links = service
			.repository
			.get_content_links_from(saved_source.nutty_id())
			.await
			.expect("Failed to get links from source block");

		assert_eq!(links.len(), 1);
		assert_eq!(links[0].source_id, *saved_source.nutty_id());
		assert_eq!(links[0].target_id, *saved_target.nutty_id());

		// Act: Update the source block to remove the link.
		let updated_source = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Paragraph {
				markdown: "No more links".to_string(),
			},
		);

		// Act: Save the updated source block.
		let saved_updated = service
			.save_content_block(updated_source.clone())
			.await
			.expect("Failed to save updated source block");

		// Assert: The orphaned link was deleted.
		let links = service
			.repository
			.get_content_links_from(saved_updated.nutty_id())
			.await
			.expect("Failed to get links from updated source block");

		assert!(links.is_empty());
	}

	#[tokio::test]
	async fn test_check_content_block_access_direct_access() {
		// Test that a user with direct access to a block can access it.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.).
		setup_test_data(&pool).await;

		// Create test navigator in the database.
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create the block in the database.
		let content_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save test block");

		// Grant direct access to the navigator using the correct permission.
		service
			.access_service
			.grant_resource_role(
				&navigator_id,
				"viewer",
				"content_block",
				content_block.nutty_id(),
			)
			.await
			.expect("Failed to grant resource role");

		// Test that the navigator can access the block.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check access");

		assert!(
			has_access,
			"Navigator should have direct access to the block"
		);

		// Cleanup.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to cleanup test block");

		// Cleanup navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_access_ancestor_access() {
		// Test that a user with access to an ancestor can access descendant blocks.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.).
		setup_test_data(&pool).await;

		// Create test navigator in the database.
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a hierarchy: parent -> child -> grandchild.
		let parent_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Parent Page".to_string(),
			},
		);

		let child_block = ContentBlock::now(
			Some(*parent_block.nutty_id()),
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Child Page".to_string(),
			},
		);

		let grandchild_block = ContentBlock::now(
			Some(*child_block.nutty_id()),
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Grandchild Page".to_string(),
			},
		);

		// Save all blocks.
		service
			.repository
			.upsert_content_block(parent_block.clone())
			.await
			.expect("Failed to save parent block");

		service
			.repository
			.upsert_content_block(child_block.clone())
			.await
			.expect("Failed to save child block");

		service
			.repository
			.upsert_content_block(grandchild_block.clone())
			.await
			.expect("Failed to save grandchild block");

		// Grant access to the parent block only.
		service
			.access_service
			.grant_resource_role(
				&navigator_id,
				"viewer",
				"content_block",
				parent_block.nutty_id(),
			)
			.await
			.expect("Failed to grant parent access");

		// Test that the navigator can access the grandchild through ancestor access.
		let grandchild_id = DissociatedNuttyId::new(&grandchild_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_access(&navigator_id, &grandchild_id)
			.await
			.expect("Failed to check access");

		assert!(
			has_access,
			"Navigator should have access to grandchild through ancestor"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(
				&DissociatedNuttyId::new(&grandchild_block.nutty_id().nid()).unwrap(),
			)
			.await
			.expect("Failed to cleanup grandchild");
		service
			.repository
			.delete_content_block(&DissociatedNuttyId::new(&child_block.nutty_id().nid()).unwrap())
			.await
			.expect("Failed to cleanup child");
		service
			.repository
			.delete_content_block(&DissociatedNuttyId::new(&parent_block.nutty_id().nid()).unwrap())
			.await
			.expect("Failed to cleanup parent");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_access_global_permission() {
		// Test that a user with global read permission can access any block.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.).
		setup_test_data(&pool).await;

		// Create test navigator in the database.
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create the block in the database.
		let content_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save test block");

		// Grant global read permission.
		service
			.access_service
			.grant_global_role(&navigator_id, "admin")
			.await
			.expect("Failed to grant global role");

		// Test that the navigator can access the block.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check access");

		assert!(
			has_access,
			"Navigator should have global access to the block"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to cleanup test block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_access_ownership() {
		// Test that a user with ownership permission can access their own blocks.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.).
		setup_test_data(&pool).await;

		// Create test navigator in the database.
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a block owned by the navigator.
		let content_block = ContentBlock::now_with_owner(
			None,
			navigator_id,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Owned Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save owned block");

		// Grant ownership permission.
		service
			.access_service
			.grant_global_role(&navigator_id, "block_owner")
			.await
			.expect("Failed to grant ownership role");

		// Test that the navigator can access their own block.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check access");

		assert!(
			has_access,
			"Navigator should have access to their own block"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to cleanup owned block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_access_no_access() {
		// Test that a user without any permissions cannot access blocks.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.).
		setup_test_data(&pool).await;

		// Create test navigator in the database.
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create the block in the database.
		let content_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save test block");

		// Test that the navigator cannot access the block (no permissions granted).
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check access");

		assert!(
			!has_access,
			"Navigator should not have access without permissions"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to cleanup test block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_access_ownership_without_permission() {
		// Test that a user who owns a block, but doesn't have ownership permission, cannot access it.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.).
		setup_test_data(&pool).await;

		// Create test navigator in the database.
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a block owned by the navigator.
		let content_block = ContentBlock::now_with_owner(
			None,
			navigator_id,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Owned Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save owned block");

		// Don't grant any ownership permissions.

		// Test that the navigator cannot access their own block without ownership permission.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check access");

		assert!(
			!has_access,
			"Navigator should not have access without ownership permission"
		);

		// Cleanup.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to cleanup owned block");

		// Cleanup navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_access_nonexistent_block() {
		// Test that checking access for a non-existent block returns an error.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.).
		setup_test_data(&pool).await;

		// Create test navigator in the database.
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database.
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a valid but non-existent block ID.
		let nonexistent_block_id = DissociatedNuttyId::new("abcdefg").unwrap();

		// Test that checking access for a non-existent block returns an error.
		let result = service
			.check_content_block_access(&navigator_id, &nonexistent_block_id)
			.await;

		assert!(
			result.is_err(),
			"Should return error for non-existent block"
		);
		match result {
			Err(ContentServiceError::FetchContentBlock(_)) => {
				// Expected error.
			}
			_ => {
				panic!("Expected FetchContentBlock error for non-existent block");
			}
		}

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_write_access_global_permission() {
		// Test that a user with global write permission can write any block.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.)
		setup_test_data(&pool).await;

		// Create test navigator in the database
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create the block in the database
		let content_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save test block");

		// Grant global write permission.
		service
			.access_service
			.grant_global_role(&navigator_id, "admin")
			.await
			.expect("Failed to grant global role");

		// Test that the navigator can write the block.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_write_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check write access");

		assert!(
			has_access,
			"Navigator should have global write access to the block"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to clean up test block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_write_access_direct_access() {
		// Test that a user with direct write access to a block can write it.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.)
		setup_test_data(&pool).await;

		// Create test navigator in the database
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create the block in the database
		let content_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save test block");

		// Grant direct write access to the navigator.
		service
			.access_service
			.grant_resource_role(
				&navigator_id,
				"editor",
				"content_block",
				content_block.nutty_id(),
			)
			.await
			.expect("Failed to grant resource role");

		// Test that the navigator can write the block.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_write_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check write access");

		assert!(
			has_access,
			"Navigator should have direct write access to the block"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to clean up test block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_write_access_ownership() {
		// Test that a user with ownership write permission can write their own blocks.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.)
		setup_test_data(&pool).await;

		// Create test navigator in the database
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a block owned by the navigator
		let content_block = ContentBlock::now_with_owner(
			None,
			navigator_id,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Owned Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save owned block");

		// Grant ownership write permission.
		service
			.access_service
			.grant_global_role(&navigator_id, "block_owner")
			.await
			.expect("Failed to grant ownership role");

		// Test that the navigator can write their own block.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_write_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check write access");

		assert!(
			has_access,
			"Navigator should have write access to their own block"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to clean up owned block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_write_access_ancestor_access() {
		// Test that a user with write access to an ancestor can write descendant blocks.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.)
		setup_test_data(&pool).await;

		// Create test navigator in the database
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a hierarchy: parent -> child -> grandchild
		let parent_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Parent Page".to_string(),
			},
		);

		let child_block = ContentBlock::now(
			Some(*parent_block.nutty_id()),
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Child Page".to_string(),
			},
		);

		let grandchild_block = ContentBlock::now(
			Some(*child_block.nutty_id()),
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Grandchild Page".to_string(),
			},
		);

		// Save all blocks
		service
			.repository
			.upsert_content_block(parent_block.clone())
			.await
			.expect("Failed to save parent block");

		service
			.repository
			.upsert_content_block(child_block.clone())
			.await
			.expect("Failed to save child block");

		service
			.repository
			.upsert_content_block(grandchild_block.clone())
			.await
			.expect("Failed to save grandchild block");

		// Grant write access to the parent block only.
		service
			.access_service
			.grant_resource_role(
				&navigator_id,
				"editor",
				"content_block",
				parent_block.nutty_id(),
			)
			.await
			.expect("Failed to grant parent write access");

		// Test that the navigator can write the grandchild through ancestor access.
		let grandchild_id = DissociatedNuttyId::new(&grandchild_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_write_access(&navigator_id, &grandchild_id)
			.await
			.expect("Failed to check write access");

		assert!(
			has_access,
			"Navigator should have write access to grandchild through ancestor"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(
				&DissociatedNuttyId::new(&grandchild_block.nutty_id().nid()).unwrap(),
			)
			.await
			.expect("Failed to clean up grandchild");
		service
			.repository
			.delete_content_block(&DissociatedNuttyId::new(&child_block.nutty_id().nid()).unwrap())
			.await
			.expect("Failed to clean up child");
		service
			.repository
			.delete_content_block(&DissociatedNuttyId::new(&parent_block.nutty_id().nid()).unwrap())
			.await
			.expect("Failed to clean up parent");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_write_access_no_access() {
		// Test that a user without any write permissions cannot write blocks.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.)
		setup_test_data(&pool).await;

		// Create test navigator in the database
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create the block in the database
		let content_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save test block");

		// Test that the navigator cannot write the block (no permissions granted).
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_write_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check write access");

		assert!(
			!has_access,
			"Navigator should not have write access without permissions"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to clean up test block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_write_access_ownership_without_permission() {
		// Test that a user who owns a block but doesn't have ownership permission cannot write it.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.)
		setup_test_data(&pool).await;

		// Create test navigator in the database
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a block owned by the navigator
		let content_block = ContentBlock::now_with_owner(
			None,
			navigator_id,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Owned Page".to_string(),
			},
		);

		service
			.repository
			.upsert_content_block(content_block.clone())
			.await
			.expect("Failed to save owned block");

		// Don't grant any ownership permissions.

		// Test that the navigator cannot write their own block without ownership permission.
		let block_id_dissociated = DissociatedNuttyId::new(&content_block.nutty_id().nid()).unwrap();
		let has_access = service
			.check_content_block_write_access(&navigator_id, &block_id_dissociated)
			.await
			.expect("Failed to check write access");

		assert!(
			!has_access,
			"Navigator should not have write access without ownership permission"
		);

		// Clean up.
		service
			.repository
			.delete_content_block(&block_id_dissociated)
			.await
			.expect("Failed to clean up owned block");

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to cleanup test navigator");
	}

	#[tokio::test]
	async fn test_check_content_block_write_access_nonexistent_block() {
		// Test that checking write access for a non-existent block returns an error.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool.clone());
		let access_repo = AccessRepository::new(pool.clone());
		let access_service = AccessService::new(access_repo);
		let service = ContentService::new(repo, access_service);

		// Set up test data (permissions, roles, etc.)
		setup_test_data(&pool).await;

		// Create test navigator in the database
		let navigator_id = NuttyId::now();
		let navigator_name = format!("test_navigator_{}", navigator_id.nid());

		// Insert navigator into database
		sqlx::query!(
			r#"
				INSERT INTO auth.navigators (id, nutty_id, name, pass, created_at, updated_at)
				VALUES ($1, $2, $3, 'test_pass', NOW(), NOW())
			"#,
			navigator_id.uuid(),
			navigator_id.nid(),
			navigator_name,
		)
		.execute(&pool)
		.await
		.expect("Failed to create test navigator");

		// Create a valid but non-existent block ID
		let nonexistent_block_id = DissociatedNuttyId::new("abcdefg").unwrap();

		// Test that checking write access for a non-existent block returns an error.
		let result = service
			.check_content_block_write_access(&navigator_id, &nonexistent_block_id)
			.await;

		assert!(
			result.is_err(),
			"Should return error for non-existent block"
		);
		match result {
			Err(ContentServiceError::FetchContentBlock(_)) => {
				// Expected error
			}
			_ => {
				panic!("Expected FetchContentBlock error for non-existent block");
			}
		}

		// Clean up navigator.
		sqlx::query!(
			r#"DELETE FROM auth.navigators WHERE id = $1"#,
			navigator_id.uuid()
		)
		.execute(&pool)
		.await
		.expect("Failed to clean up test navigator");
	}

	// Helper function to set up test data.
	async fn setup_test_data(pool: &sqlx::PgPool) {
		// Insert test permissions.
		sqlx::query!(
			r#"
				INSERT INTO auth.permissions (name, description)
				VALUES
					('content_blocks:read:all', 'Read all content blocks'),
					('content_blocks:write:all', 'Write all content blocks'),
					('content_blocks:write:own', 'Write own content blocks'),
					('content_blocks:read:resource', 'Read specific content block'),
					('content_blocks:read:own', 'Read own content blocks'),
					('content_blocks:write', 'Write specific content block')
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
					('editor', 'content_blocks:write'),
					('viewer', 'content_blocks:read:resource'),
					('block_owner', 'content_blocks:read:own'),
					('block_owner', 'content_blocks:write:own')
				ON CONFLICT (role_name, permission_name) DO NOTHING
			"#
		)
		.execute(pool)
		.await
		.expect("Failed to insert role permissions");
	}
}
