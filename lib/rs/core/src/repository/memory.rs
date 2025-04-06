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
		let blocks = self.blocks.read().await;
		Ok(blocks.get(&id).cloned())
	}

	async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError> {
		let mut blocks = self.blocks.write().await;
		blocks.insert(content_block.id, content_block.clone());
		Ok(content_block)
	}

	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError> {
		let mut blocks = self.blocks.write().await;
		blocks.remove(&id);
		Ok(())
	}

	async fn link_repository(
		&mut self,
		linked_repository: Arc<dyn ContentRepository>,
	) -> Result<(), ApiError> {
		let mut linked = self.linked_repository.write().await;
		*linked = Some(linked_repository);
		Ok(())
	}

	async fn is_linked(&self) -> bool {
		let linked = self.linked_repository.read().await;
		linked.is_some()
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
	async fn test_repository_linking() {
		// Arrange: Create two memory repositories.
		let mut repository = MemoryContentRepository::new();
		let another_repository = Arc::new(MemoryContentRepository::new());

		// Act: Link the repositories.
		repository
			.link_repository(another_repository.clone())
			.await
			.expect("Failed to link repositories");

		// Assert: The repositories are linked.
		assert!(repository.is_linked().await);
	}
}
