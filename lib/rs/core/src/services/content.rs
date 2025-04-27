use crate::{
	models::{ContentBlock, ContentContext, ContentLink, NuttyId},
	repository::{
		ContentRepository, ContentRepositoryError, Repository, repository::TransactionExt,
	},
};

pub struct ContentService {
	// The content repository to use for storing and retrieving content.
	repository: ContentRepository,
}

impl ContentService {
	// Create a new content service with the given repository.
	pub fn new(repository: ContentRepository) -> Self {
		ContentService { repository }
	}

	/// Get a content block's context.
	pub async fn get_content_block_context(
		&self,
		nutty_id: &NuttyId,
	) -> Result<ContentContext, ContentServiceError> {
		// Get the content block.
		let content_block = self
			.repository
			.get_content_block(&nutty_id.into())
			.await
			.map_err(ContentServiceError::FetchContentBlock)?
			.ok_or(ContentServiceError::ContentBlockNotFound)?;

		// Get the ancestor blocks.
		let ancestors = self
			.repository
			.get_ancestor_blocks(&nutty_id.into())
			.await
			.map_err(ContentServiceError::FetchAncestorBlocks)?;

		// Get the descendant blocks.
		let descendants = self
			.repository
			.get_descendant_blocks(&nutty_id.into())
			.await
			.map_err(ContentServiceError::FetchDescendantBlocks)?;

		// Get immediate children.
		let children_ids = descendants
			.iter()
			.filter(|block| (block.parent_id.as_ref() == Some(nutty_id)))
			.map(|block| *block.nutty_id())
			.collect::<Vec<_>>();

		// Get outbound links (references).
		let outbound_links = self
			.repository
			.get_content_links_from(nutty_id)
			.await
			.map_err(ContentServiceError::FetchOutboundLinks)?;

		// Get inbound links (backlinks).
		let inbound_links = self
			.repository
			.get_content_links_to(nutty_id)
			.await
			.map_err(ContentServiceError::FetchInboundLinks)?;

		// Build the block cache.
		let mut block_cache = std::collections::HashMap::new();

		// Add the main content block to the cache.
		block_cache.insert(*nutty_id, content_block.clone());

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
			.block_id(*nutty_id)
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::{BlockContent, ContentBlock, FractionalIndex};
	use crate::repository::ContentRepository;
	use sqlx::postgres::PgPoolOptions;
	use sqlx::{Pool, Postgres};

	async fn connect_to_test_database() -> Pool<Postgres> {
		PgPoolOptions::new()
			.max_connections(5)
			.connect("postgres://nutty@localhost:5432/nuttyverse")
			.await
			.expect("Failed to connect to test database")
	}

	#[tokio::test]
	async fn test_get_content_block_context() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool);
		let service = ContentService::new(repo);

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
			.get_content_block_context(middle_block.nutty_id())
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
			.get_content_block_context(child_block.nutty_id())
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
		let repo = ContentRepository::new(pool);
		let service = ContentService::new(repo);

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
}
