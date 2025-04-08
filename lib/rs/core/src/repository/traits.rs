use crate::errors::ApiError;
use crate::models::{ContentBlock, ContentLink};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// A repository for content blocks and links.
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

	/// Get a content link by its identifier.
	async fn get_content_link(&self, id: Uuid) -> Result<Option<ContentLink>, ApiError>;

	/// Get all content links from a content block.
	async fn get_content_links_from(&self, id: Uuid) -> Result<Vec<ContentLink>, ApiError>;

	/// Get all content links to a content block.
	async fn get_content_links_to(&self, id: Uuid) -> Result<Vec<ContentLink>, ApiError>;

	/// Save a content link between two content blocks.
	async fn save_content_link(&self, link: ContentLink) -> Result<(), ApiError>;

	/// Delete a content link between two content blocks.
	async fn delete_content_link(&self, link: ContentLink) -> Result<(), ApiError>;

	/// Check if two blocks are linked.
	async fn are_blocks_linked(&self, source_id: Uuid, target_id: Uuid) -> Result<bool, ApiError>;

	/// Link this repository with another repository.
	async fn link_repository(
		&mut self,
		repository: Arc<dyn ContentRepository>,
	) -> Result<(), ApiError>;
}
