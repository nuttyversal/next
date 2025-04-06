use crate::errors::ApiError;
use crate::models::ContentBlock;
use crate::repository::traits::ContentRepository;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A repository that stores content blocks in memory.
pub struct MemoryContentRepository {
	/// The blocks in this repository.
	blocks: RwLock<HashMap<Uuid, ContentBlock>>,

	/// A linked repository — used to connect to another repository to sync
	/// content blocks during fetching and saving operations.
	linked_repository: RwLock<Option<Arc<dyn ContentRepository>>>,
}

impl MemoryContentRepository {
	/// Create a new memory repository.
	pub fn new() -> Self {
		Self {
			blocks: RwLock::new(HashMap::new()),
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
		let blocks = self.blocks.read().await;
		if let Some(block) = blocks.get(&id) {
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
		// Save the content block to memory.
		let mut blocks = self.blocks.write().await;
		blocks.insert(content_block.id, content_block.clone());

		// Sync content block to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.save_content_block(content_block).await,
			None => Ok(content_block),
		}
	}

	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError> {
		// Delete the content block from memory.
		let mut blocks = self.blocks.write().await;
		blocks.remove(&id);

		// Delete the content block from the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.delete_content_block(id).await,
			None => Ok(()),
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
}
