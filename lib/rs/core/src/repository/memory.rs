use crate::errors::ApiError;
use crate::models::{ContentBlock, ContentLink};
use crate::repository::traits::ContentRepository;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A repository that stores content blocks in memory.
pub struct MemoryContentRepository {
	/// The internal state of the repository, protected by a single RwLock.
	state: RwLock<RepositoryState>,

	/// A linked repository — used to connect to another repository to sync
	/// content blocks during fetching and saving operations.
	linked_repository: RwLock<Option<Arc<dyn ContentRepository>>>,
}

/// The internal state of the repository.
struct RepositoryState {
	/// The content blocks in this repository.
	blocks: HashMap<Uuid, ContentBlock>,

	/// The content links in this repository.
	links: HashMap<Uuid, ContentLink>,

	/// Maps each block ID to the content links where it is the source.
	/// This is used to quickly look up all the links for a given block.
	source_block_links: HashMap<Uuid, HashSet<Uuid>>,

	/// Maps each block ID to the content links where it is the target.
	/// This is used to quickly look up all the links for a given block.
	target_block_links: HashMap<Uuid, HashSet<Uuid>>,
}

impl MemoryContentRepository {
	/// Create a new memory repository.
	pub fn new() -> Self {
		Self {
			state: RwLock::new(RepositoryState {
				blocks: HashMap::new(),
				links: HashMap::new(),
				source_block_links: HashMap::new(),
				target_block_links: HashMap::new(),
			}),

			linked_repository: RwLock::new(None),
		}
	}
}

impl Default for MemoryContentRepository {
	fn default() -> Self {
		Self::new()
	}
}

#[async_trait]
impl ContentRepository for MemoryContentRepository {
	async fn get_content_block(&self, id: Uuid) -> Result<Option<ContentBlock>, ApiError> {
		// Try to find the content block in memory.
		let state = self.state.read().await;

		if let Some(block) = state.blocks.get(&id) {
			return Ok(Some(block.clone()));
		}

		// Try to find the content block in the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.get_content_block(id).await,
			None => Ok(None),
		}
	}

	async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError> {
		{
			// Save the content block to memory.
			let mut state = self.state.write().await;
			state.blocks.insert(content_block.id, content_block.clone());
		}

		// Sync content block to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.save_content_block(content_block).await,
			None => Ok(content_block),
		}
	}

	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError> {
		{
			// Delete the content block from memory.
			let mut state = self.state.write().await;
			state.blocks.remove(&id);
		}

		// Delete the content block from the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.delete_content_block(id).await,
			None => Ok(()),
		}
	}

	async fn get_content_link(&self, id: Uuid) -> Result<Option<ContentLink>, ApiError> {
		let state = self.state.read().await;

		// Try to find the content link in memory.
		if let Some(link) = state.links.get(&id) {
			return Ok(Some(*link));
		}

		// Try to find the content link in the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.get_content_link(id).await,
			None => Ok(None),
		}
	}

	async fn get_content_links_from(&self, id: Uuid) -> Result<Vec<ContentLink>, ApiError> {
		let state = self.state.read().await;

		// Get all link IDs for this source block.
		let link_ids = state
			.source_block_links
			.get(&id)
			.map(|targets| targets.iter().copied().collect::<Vec<_>>())
			.unwrap_or_default();

		// Get all links from memory.
		let mut links = std::collections::HashMap::new();

		for link_id in link_ids.iter() {
			if let Some(link) = state.links.get(link_id) {
				links.insert(link.id, *link);
			}
		}

		// Try to get additional links from the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => {
				let linked_links = linked_repository.get_content_links_from(id).await?;

				for link in linked_links {
					links.insert(link.id, link);
				}

				Ok(links.into_values().collect())
			}

			None => Ok(links.into_values().collect()),
		}
	}

	async fn get_content_links_to(&self, id: Uuid) -> Result<Vec<ContentLink>, ApiError> {
		let state = self.state.read().await;

		// Get all link IDs for this target block.
		let link_ids = state
			.target_block_links
			.get(&id)
			.map(|sources| sources.iter().copied().collect::<Vec<_>>())
			.unwrap_or_default();

		// Get all links from memory.
		let mut links = std::collections::HashMap::new();

		for link_id in link_ids.iter() {
			if let Some(link) = state.links.get(link_id) {
				links.insert(link.id, *link);
			}
		}

		// Try to get additional links from the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => {
				let linked_links = linked_repository.get_content_links_to(id).await?;

				for link in linked_links {
					links.insert(link.id, link);
				}

				Ok(links.into_values().collect())
			}

			None => Ok(links.into_values().collect()),
		}
	}

	async fn save_content_link(&self, link: ContentLink) -> Result<(), ApiError> {
		{
			let mut state = self.state.write().await;

			// Save the content link to memory.
			state.links.insert(link.id, link);

			// Update the block links.
			state
				.source_block_links
				.entry(link.source_id)
				.or_insert_with(HashSet::new)
				.insert(link.id);

			state
				.target_block_links
				.entry(link.target_id)
				.or_insert_with(HashSet::new)
				.insert(link.id);
		}

		// Sync content link to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.save_content_link(link).await,
			None => Ok(()),
		}
	}

	async fn delete_content_link(&self, link: ContentLink) -> Result<(), ApiError> {
		{
			let mut state = self.state.write().await;

			// Delete the content link from memory.
			state.links.remove(&link.id);

			// Update the block links.
			if let Some(links) = state.source_block_links.get_mut(&link.source_id) {
				links.remove(&link.id);
				if links.is_empty() {
					state.source_block_links.remove(&link.source_id);
				}
			}

			if let Some(links) = state.target_block_links.get_mut(&link.target_id) {
				links.remove(&link.id);
				if links.is_empty() {
					state.target_block_links.remove(&link.target_id);
				}
			}
		}

		// Sync content link to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.delete_content_link(link).await,
			None => Ok(()),
		}
	}

	async fn are_blocks_linked(&self, source_id: Uuid, target_id: Uuid) -> Result<bool, ApiError> {
		let state = self.state.read().await;

		// Check if there's a link from source to target in memory
		let has_link = state
			.source_block_links
			.get(&source_id)
			.map_or(false, |link_ids| {
				link_ids.iter().any(|link_id| {
					state
						.links
						.get(link_id)
						.map_or(false, |link| link.target_id == target_id)
				})
			});

		if has_link {
			return Ok(true);
		}

		// Check the linked repository if no link was found
		match &*self.linked_repository.read().await {
			Some(linked_repository) => {
				linked_repository
					.are_blocks_linked(source_id, target_id)
					.await
			}
			None => Ok(false),
		}
	}

	async fn link_repository(
		&mut self,
		linked_repository: Arc<dyn ContentRepository>,
	) -> Result<(), ApiError> {
		*self.linked_repository.write().await = Some(linked_repository);
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::BlockContent;

	#[tokio::test]
	async fn test_content_block_operations() {
		// Arrange: Create a memory repository.
		let repo = MemoryContentRepository::new();

		// Arrange: Create a test content block.
		let test_content_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		// Act: Save the test content block.
		let saved = repo
			.save_content_block(test_content_block.clone())
			.await
			.expect("Failed to save content block");

		// Assert: The saved content block matches the original.
		assert_eq!(saved.id, test_content_block.id);
		assert_eq!(saved.parent_id, test_content_block.parent_id);
		assert!(matches!(saved.content, BlockContent::Page { title } if title == "Test Page"));

		// Act: Query the content block.
		let retrieved = repo
			.get_content_block(test_content_block.id)
			.await
			.expect("Failed to get content block")
			.expect("Content block not found");

		// Assert: The retrieved content block matches the original.
		assert_eq!(retrieved.id, test_content_block.id);
		assert_eq!(retrieved.parent_id, test_content_block.parent_id);
		assert!(matches!(retrieved.content, BlockContent::Page { title } if title == "Test Page"));

		// Act: Update the content block.
		let updated_content_block = ContentBlock::new(
			test_content_block.id,
			test_content_block.parent_id,
			BlockContent::Page {
				title: "Updated Page".to_string(),
			},
		);

		let updated = repo
			.save_content_block(updated_content_block)
			.await
			.expect("Failed to update content block");

		// Assert: The content block was updated.
		assert_eq!(updated.id, test_content_block.id);
		assert_eq!(updated.parent_id, test_content_block.parent_id);
		assert!(matches!(updated.content, BlockContent::Page { title } if title == "Updated Page"));

		// Act: Delete the content block.
		repo
			.delete_content_block(test_content_block.id)
			.await
			.expect("Failed to delete content block");

		// Assert: The content block no longer exists.
		let retrieved = repo.get_content_block(test_content_block.id).await.unwrap();
		assert!(retrieved.is_none());
	}

	#[tokio::test]
	async fn test_linked_repository_operations() {
		// Arrange: Create repositories.
		let mut primary_repo = MemoryContentRepository::new();
		let secondary_repo = Arc::new(MemoryContentRepository::new());

		// Act: Link the repositories.
		primary_repo
			.link_repository(secondary_repo.clone())
			.await
			.expect("Failed to link repositories");

		// Arrange: Create a test content block.
		let test_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Linked Test Page".to_string(),
			},
		);

		// Act: Save to primary repository, which should also sync to secondary.
		primary_repo
			.save_content_block(test_block.clone())
			.await
			.expect("Failed to save content block");

		// Assert: Block exists in both repositories.
		let primary_block = primary_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from primary repository")
			.expect("Block not found in primary repository");

		let secondary_block = secondary_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from secondary repository")
			.expect("Block not found in secondary repository");

		assert_eq!(primary_block.id, secondary_block.id);
		assert_eq!(primary_block.parent_id, secondary_block.parent_id);
		assert!(matches!(
			primary_block.content,
			BlockContent::Page { title } if title == "Linked Test Page"
		));

		// Act: Update in primary repository.
		let updated_block = ContentBlock::new(
			test_block.id,
			test_block.parent_id,
			BlockContent::Page {
				title: "Updated Linked Page".to_string(),
			},
		);

		primary_repo
			.save_content_block(updated_block)
			.await
			.expect("Failed to update in primary repository");

		// Assert: Update synced to secondary repository.
		let secondary_block = secondary_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from secondary repository")
			.expect("Block not found in secondary repository");

		// Assert: The content block was updated in the secondary repository.
		assert!(matches!(
			secondary_block.content,
			BlockContent::Page { title } if title == "Updated Linked Page"
		));

		// Act: Delete from primary repository.
		primary_repo
			.delete_content_block(test_block.id)
			.await
			.expect("Failed to delete from primary repository");

		// Assert: Deletion synced to secondary repository.
		let secondary_block = secondary_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from secondary repository");

		// Assert: The content block no longer exists in the secondary repository.
		assert!(secondary_block.is_none());
	}

	#[tokio::test]
	async fn test_content_link_operations() {
		// Arrange: Create a memory repository.
		let repo = MemoryContentRepository::new();

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Target Page".to_string(),
			},
		);

		// Act: Save the content blocks.
		repo
			.save_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		repo
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a content link between the blocks.
		let link = ContentLink::now(source_block.id, target_block.id);

		repo
			.save_content_link(link)
			.await
			.expect("Failed to create content link");

		// Assert: The blocks are linked.
		assert!(
			repo
				.are_blocks_linked(source_block.id, target_block.id)
				.await
				.expect("Failed to check link")
		);

		// Act: Delete the link.
		repo
			.delete_content_link(link)
			.await
			.expect("Failed to delete content link");

		// Assert: The blocks are no longer linked.
		assert!(
			!repo
				.are_blocks_linked(source_block.id, target_block.id)
				.await
				.expect("Failed to check link")
		);
	}

	#[tokio::test]
	async fn test_linked_repository_content_links() {
		// Arrange: Create repositories.
		let mut primary_repo = MemoryContentRepository::new();
		let secondary_repo = Arc::new(MemoryContentRepository::new());

		// Act: Link the repositories.
		primary_repo
			.link_repository(secondary_repo.clone())
			.await
			.expect("Failed to link repositories");

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Target Page".to_string(),
			},
		);

		// Act: Save the content blocks.
		primary_repo
			.save_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		primary_repo
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a content link in primary repository.
		let link = ContentLink::now(source_block.id, target_block.id);

		primary_repo
			.save_content_link(link)
			.await
			.expect("Failed to create content link");

		// Assert: The blocks are linked in both repositories.
		assert!(
			primary_repo
				.are_blocks_linked(source_block.id, target_block.id)
				.await
				.expect("Failed to check link in primary repository")
		);

		assert!(
			secondary_repo
				.are_blocks_linked(source_block.id, target_block.id)
				.await
				.expect("Failed to check link in secondary repository")
		);

		// Act: Delete the link from primary repository.
		primary_repo
			.delete_content_link(link)
			.await
			.expect("Failed to delete content link");

		// Assert: The blocks are no longer linked in either repository.
		assert!(
			!primary_repo
				.are_blocks_linked(source_block.id, target_block.id)
				.await
				.expect("Failed to check link in primary repository")
		);

		assert!(
			!secondary_repo
				.are_blocks_linked(source_block.id, target_block.id)
				.await
				.expect("Failed to check link in secondary repository")
		);
	}

	#[tokio::test]
	async fn test_content_link_query_operations() {
		// Arrange: Create a memory repository.
		let repo = MemoryContentRepository::new();

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Target Page".to_string(),
			},
		);

		// Act: Save the content blocks.
		repo
			.save_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		repo
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a content link between the blocks.
		let link = ContentLink::now(source_block.id, target_block.id);

		repo
			.save_content_link(link)
			.await
			.expect("Failed to create content link");

		// Act: Get the content link by ID.
		let retrieved_link = repo
			.get_content_link(link.id)
			.await
			.expect("Failed to get content link")
			.expect("Content link not found");

		// Assert: The retrieved link matches the original.
		assert_eq!(retrieved_link.id, link.id);
		assert_eq!(retrieved_link.source_id, source_block.id);
		assert_eq!(retrieved_link.target_id, target_block.id);

		// Act: Get all links from the source block.
		let links_from = repo
			.get_content_links_from(source_block.id)
			.await
			.expect("Failed to get links from source block");

		// Assert: The links from source block match the original.
		assert_eq!(links_from.len(), 1);
		assert_eq!(links_from[0].id, link.id);
		assert_eq!(links_from[0].source_id, source_block.id);
		assert_eq!(links_from[0].target_id, target_block.id);

		// Act: Get all links to the target block.
		let links_to = repo
			.get_content_links_to(target_block.id)
			.await
			.expect("Failed to get links to target block");

		// Assert: The links to target block match the original.
		assert_eq!(links_to.len(), 1);
		assert_eq!(links_to[0].id, link.id);
		assert_eq!(links_to[0].source_id, source_block.id);
		assert_eq!(links_to[0].target_id, target_block.id);

		// Act: Try to get a non-existent link.
		let non_existent_link = repo
			.get_content_link(Uuid::now_v7())
			.await
			.expect("Failed to check non-existent link");

		// Assert: No link is found.
		assert!(non_existent_link.is_none());

		// Act: Try to get links from a non-existent source.
		let no_links_from = repo
			.get_content_links_from(Uuid::now_v7())
			.await
			.expect("Failed to get links from non-existent source");

		// Assert: No links are found.
		assert!(no_links_from.is_empty());

		// Act: Try to get links to a non-existent target.
		let no_links_to = repo
			.get_content_links_to(Uuid::now_v7())
			.await
			.expect("Failed to get links to non-existent target");

		// Assert: No links are found.
		assert!(no_links_to.is_empty());
	}

	#[tokio::test]
	async fn test_linked_repository_content_link_queries() {
		// Arrange: Create repositories.
		let mut primary_repo = MemoryContentRepository::new();
		let secondary_repo = Arc::new(MemoryContentRepository::new());

		// Act: Link the repositories.
		primary_repo
			.link_repository(secondary_repo.clone())
			.await
			.expect("Failed to link repositories");

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Target Page".to_string(),
			},
		);

		// Act: Save the content blocks.
		primary_repo
			.save_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		primary_repo
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a content link in primary repository.
		let link = ContentLink::now(source_block.id, target_block.id);

		primary_repo
			.save_content_link(link)
			.await
			.expect("Failed to create content link");

		// Act: Get the link from both repositories.
		let primary_link = primary_repo
			.get_content_link(link.id)
			.await
			.expect("Failed to get link from primary repository")
			.expect("Link not found in primary repository");

		let secondary_link = secondary_repo
			.get_content_link(link.id)
			.await
			.expect("Failed to get link from secondary repository")
			.expect("Link not found in secondary repository");

		// Assert: The links match in both repositories.
		assert_eq!(primary_link.id, secondary_link.id);
		assert_eq!(primary_link.source_id, secondary_link.source_id);
		assert_eq!(primary_link.target_id, secondary_link.target_id);

		// Act: Get all links from source block in both repositories.
		let primary_links_from = primary_repo
			.get_content_links_from(source_block.id)
			.await
			.expect("Failed to get links from primary repository");

		let secondary_links_from = secondary_repo
			.get_content_links_from(source_block.id)
			.await
			.expect("Failed to get links from secondary repository");

		// Assert: The links from source block match in both repositories.
		assert_eq!(primary_links_from.len(), secondary_links_from.len());
		assert_eq!(primary_links_from[0].id, secondary_links_from[0].id);

		// Act: Get all links to target block in both repositories.
		let primary_links_to = primary_repo
			.get_content_links_to(target_block.id)
			.await
			.expect("Failed to get links to primary repository");

		let secondary_links_to = secondary_repo
			.get_content_links_to(target_block.id)
			.await
			.expect("Failed to get links to secondary repository");

		// Assert: The links to target block match in both repositories.
		assert_eq!(primary_links_to.len(), secondary_links_to.len());
		assert_eq!(primary_links_to[0].id, secondary_links_to[0].id);
	}
}
