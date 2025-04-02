use crate::errors::ApiError;
use crate::models::ContentBlock;
use async_trait::async_trait;
use uuid::Uuid;

/// A repository for content blocks.
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
}
