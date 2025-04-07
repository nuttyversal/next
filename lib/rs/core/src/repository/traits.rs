use crate::errors::ApiError;
use crate::models::{ContentBlock, ContentLink};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// A repository for content blocks.
///
/// If a repository is linked with another repository, it will be able to sync
/// content blocks & links to and from the linked repository.
#[async_trait]
pub trait ContentRepository: Send + Sync {
	/// Get a block of content by its identifier.
	async fn get_content_block(&self, id: Uuid) -> Result<Option<ContentBlock>, ApiError>;

	/// Save a block of content with upsertion semantics.
	async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError>;

	/// Delete a block of content by its identifier.
	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError>;

	/// Save a content link between two content blocks.
	async fn save_content_link(&self, link: ContentLink) -> Result<(), ApiError>;

	/// Delete a content link between two content blocks.
	async fn delete_content_link(&self, link: ContentLink) -> Result<(), ApiError>;

	/// Check if two blocks are linked.
	async fn are_blocks_linked(&self, source_id: Uuid, target_id: Uuid) -> Result<bool, ApiError>;

	/// Link this repository with another repository.
	async fn link_repository(
		&mut self,
		linked_repository: Arc<dyn ContentRepository>,
	) -> Result<(), ApiError>;
}
