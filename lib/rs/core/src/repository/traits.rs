use crate::errors::ApiError;
use crate::models::ContentBlock;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ContentRepository: Send + Sync {
	async fn get_content_block(&self, id: Uuid) -> Result<Option<ContentBlock>, ApiError>;
	async fn save_content_block(&self, content_block: ContentBlock) -> Result<ContentBlock, ApiError>;
	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError>;
}
