use crate::errors::ApiError;
use crate::models::{AnyNuttyId, ContentBlock, ContentLink, NuttyId};
use async_trait::async_trait;

/// A repository for content blocks and links.
#[async_trait]
pub trait ContentRepository: Send + Sync {
	/// Get a content block by its Nutty ID.
	async fn get_content_block(
		&self,
		nutty_id: &AnyNuttyId,
	) -> Result<Option<ContentBlock>, ApiError>;

	/// Upsert a content block.
	async fn upsert_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError>;

	/// Delete a block of content by its identifier.
	async fn delete_content_block(&self, nutty_id: &AnyNuttyId) -> Result<(), ApiError>;

	/// Get a content link by its identifier.
	async fn get_content_link(&self, id: &AnyNuttyId) -> Result<Option<ContentLink>, ApiError>;

	/// Get all content links from a content block.
	async fn get_content_links_from(&self, nutty_id: &NuttyId)
	-> Result<Vec<ContentLink>, ApiError>;

	/// Get all content links to a content block.
	async fn get_content_links_to(&self, nutty_id: &NuttyId) -> Result<Vec<ContentLink>, ApiError>;

	/// Upsert a content link between two content blocks.
	async fn upsert_content_link(&self, link: ContentLink) -> Result<ContentLink, ApiError>;

	/// Delete a content link between two content blocks.
	async fn delete_content_link(&self, link: ContentLink) -> Result<(), ApiError>;

	/// Check if two content blocks are linked.
	async fn is_linked(&self, source_id: &NuttyId, target_id: &NuttyId) -> Result<bool, ApiError>;
}
