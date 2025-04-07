use crate::errors::ApiError;
use crate::models::{ContentBlock, ContentLink};
use crate::repository::traits::ContentRepository;
use async_trait::async_trait;
use sqlx::types::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A repository for content blocks that uses a PostgreSQL database as its backing store.
pub struct PostgresContentRepository {
	/// The PostgreSQL database pool.
	pool: sqlx::PgPool,

	/// A linked repository — used to connect to another repository to sync
	/// content blocks & links during fetching and saving operations.
	linked_repository: RwLock<Option<Arc<dyn ContentRepository>>>,
}

impl PostgresContentRepository {
	/// Create a new PostgreSQL content repository.
	pub fn new(pool: sqlx::PgPool) -> Self {
		Self {
			pool,
			linked_repository: RwLock::new(None),
		}
	}
}

#[async_trait]
impl ContentRepository for PostgresContentRepository {
	async fn get_content_block(&self, id: Uuid) -> Result<Option<ContentBlock>, ApiError> {
		// Try to find the content block in the database.
		let record = sqlx::query!(
			r#"
				SELECT id, parent_id, content
				FROM blocks
				WHERE id = $1
			"#,
			id
		)
		.fetch_optional(&self.pool)
		.await?;

		match (record, &*self.linked_repository.read().await) {
			// Found the content block in the database!
			(Some(record), _) => ContentBlock::deserialize_content(record.content)
				.map_err(ApiError::from)
				.map(|content| {
					Some(ContentBlock {
						id: record.id,
						parent_id: record.parent_id,
						content,
					})
				}),

			// Try to find the content block in the linked repository.
			(_, Some(linked_repository)) => linked_repository.get_content_block(id).await,

			// Cannot find the content block anywhere.
			(_, None) => Ok(None),
		}
	}

	async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError> {
		// Save the content block to the database.
		sqlx::query!(
			r#"
				INSERT INTO blocks (id, parent_id, content)
				VALUES ($1, $2, $3)
				ON CONFLICT (id) DO UPDATE
				SET parent_id = EXCLUDED.parent_id, content = EXCLUDED.content
				RETURNING id, parent_id, content
			"#,
			content_block.id,
			content_block.parent_id,
			content_block.serialize_content()?
		)
		.fetch_one(&self.pool)
		.await?;

		// Sync content block to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.save_content_block(content_block).await,
			None => Ok(content_block),
		}
	}

	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError> {
		// Delete the content block from the database.
		sqlx::query!(
			r#"
				DELETE FROM blocks
				WHERE id = $1
			"#,
			id
		)
		.execute(&self.pool)
		.await?;

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

	async fn save_content_link(&self, link: ContentLink) -> Result<(), ApiError> {
		// Save the content link to the database.
		sqlx::query!(
			r#"
				INSERT INTO links (id, source_id, target_id)
				VALUES ($1, $2, $3)
				ON CONFLICT (id) DO NOTHING
			"#,
			link.id,
			link.source_id,
			link.target_id
		)
		.execute(&self.pool)
		.await?;

		// Sync the content link to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.save_content_link(link).await,

			None => Ok(()),
		}
	}

	async fn delete_content_link(&self, link: ContentLink) -> Result<(), ApiError> {
		// Delete the content link from the database.
		sqlx::query!(
			r#"
				DELETE FROM links
				WHERE id = $1
			"#,
			link.id
		)
		.execute(&self.pool)
		.await?;

		// Sync the deletion to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.delete_content_link(link).await,

			None => Ok(()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{models::BlockContent, repository::memory::MemoryContentRepository};
	use sqlx::PgPool;

	async fn connect_to_test_database() -> PgPool {
		PgPool::connect("postgres://nutty@localhost:5432/nuttyverse")
			.await
			.expect("Failed to create test database pool")
	}

	#[tokio::test]
	async fn test_content_block_operations() {
		// Arrange: Connect to the database.
		let database_pool = connect_to_test_database().await;
		let repository = PostgresContentRepository::new(database_pool);

		// Arrange: Create a test content block.
		let test_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		// Act: Save the test content block.
		let saved_block = repository
			.save_content_block(test_block.clone())
			.await
			.expect("Failed to save content block");

		// Assert: The saved content block matches the original.
		assert_eq!(saved_block.id, test_block.id);
		assert_eq!(saved_block.parent_id, test_block.parent_id);
		assert!(matches!(saved_block.content, BlockContent::Page { title } if title == "Test Page"));

		// Act: Query the content block.
		let retrieved_block = repository
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get content block")
			.expect("Content block not found");

		// Assert: The retrieved content block matches the original.
		assert_eq!(retrieved_block.id, test_block.id);
		assert_eq!(retrieved_block.parent_id, test_block.parent_id);
		assert!(
			matches!(retrieved_block.content, BlockContent::Page { title } if title == "Test Page")
		);

		// Act: Update the content block.
		let updated_block = ContentBlock::new(
			test_block.id,
			test_block.parent_id,
			BlockContent::Page {
				title: "Updated Page".to_string(),
			},
		);

		let updated_block = repository
			.save_content_block(updated_block)
			.await
			.expect("Failed to update content block");

		// Assert: The content block was updated.
		assert_eq!(updated_block.id, test_block.id);
		assert_eq!(updated_block.parent_id, test_block.parent_id);
		assert!(
			matches!(updated_block.content, BlockContent::Page { title } if title == "Updated Page")
		);

		// Act: Delete the content block.
		repository
			.delete_content_block(test_block.id)
			.await
			.expect("Failed to delete content block");

		// Act: Fetch the content block again.
		let retrieved_block = repository.get_content_block(test_block.id).await.unwrap();

		// Assert: The content block no longer exists.
		assert!(retrieved_block.is_none());
	}

	#[tokio::test]
	async fn test_linked_repository_operations() {
		// Arrange: Create repositories.
		let database_pool = connect_to_test_database().await;
		let mut postgres_repo = PostgresContentRepository::new(database_pool);
		let memory_repo = Arc::new(MemoryContentRepository::new());

		// Act: Link the repositories.
		postgres_repo
			.link_repository(memory_repo.clone())
			.await
			.expect("Failed to link repositories");

		// Arrange: Create a test content block.
		let test_block = ContentBlock::now(
			None,
			BlockContent::Page {
				title: "Linked Test Page".to_string(),
			},
		);

		// Act: Save to PostgreSQL, which should also sync to memory.
		postgres_repo
			.save_content_block(test_block.clone())
			.await
			.expect("Failed to save content block");

		// Assert: The content block exists in both repositories.
		let postgres_block = postgres_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from postgres")
			.expect("Block not found in postgres");

		let memory_block = memory_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from memory")
			.expect("Block not found in memory");

		assert_eq!(postgres_block.id, memory_block.id);
		assert_eq!(postgres_block.parent_id, memory_block.parent_id);
		assert!(matches!(
			postgres_block.content,
			BlockContent::Page { title } if title == "Linked Test Page"
		));

		// Act: Update in PostgreSQL, which should also sync to memory.
		let updated_block = ContentBlock::new(
			test_block.id,
			test_block.parent_id,
			BlockContent::Page {
				title: "Updated Linked Page".to_string(),
			},
		);

		postgres_repo
			.save_content_block(updated_block)
			.await
			.expect("Failed to update in postgres");

		// Assert: The content block was updated in memory.
		let memory_block = memory_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from memory")
			.expect("Block not found in memory");

		assert!(matches!(
			memory_block.content,
			BlockContent::Page { title } if title == "Updated Linked Page"
		));

		// Act: Delete from PostgreSQL, which should also sync to memory.
		postgres_repo
			.delete_content_block(test_block.id)
			.await
			.expect("Failed to delete from PostgreSQL");

		// Assert: The content block no longer exists in memory.
		let memory_block = memory_repo
			.get_content_block(test_block.id)
			.await
			.expect("Failed to get from memory");

		assert!(memory_block.is_none());
	}

	#[tokio::test]
	async fn test_content_link_operations() {
		// Arrange: Connect to the database.
		let database_pool = connect_to_test_database().await;
		let repository = PostgresContentRepository::new(database_pool);

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
		repository
			.save_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		repository
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a content link between the blocks.
		let link = ContentLink::now(source_block.id, target_block.id);

		repository
			.save_content_link(link)
			.await
			.expect("Failed to create content link");

		// Assert: The link exists in the database with an ID.
		let record = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM links
				WHERE id = $1
			"#,
			link.id
		)
		.fetch_one(&repository.pool)
		.await
		.expect("Failed to fetch link");

		assert_eq!(record.source_id, link.source_id);
		assert_eq!(record.target_id, link.target_id);

		// Act: Delete the link.
		repository
			.delete_content_link(link)
			.await
			.expect("Failed to delete content link");

		// Assert: The link no longer exists in the database.
		let link_exists = sqlx::query!(
			r#"
				SELECT EXISTS (
					SELECT 1 FROM links
					WHERE source_id = $1 AND target_id = $2
				) as "exists!"
			"#,
			source_block.id,
			target_block.id
		)
		.fetch_one(&repository.pool)
		.await
		.expect("Failed to check link existence")
		.exists;

		assert!(!link_exists);
	}

	#[tokio::test]
	async fn test_linked_repository_content_links() {
		// Arrange: Create repositories.
		let database_pool = connect_to_test_database().await;
		let mut postgres_repo = PostgresContentRepository::new(database_pool);
		let memory_repo = Arc::new(MemoryContentRepository::new());

		// Act: Link the repositories.
		postgres_repo
			.link_repository(memory_repo.clone())
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
		postgres_repo
			.save_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		postgres_repo
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a content link in PostgreSQL, which should also sync to memory.
		let link = ContentLink::now(source_block.id, target_block.id);

		postgres_repo
			.save_content_link(link)
			.await
			.expect("Failed to create content link");

		// Assert: The link exists in both repositories.
		let postgres_link = sqlx::query!(
			r#"
				SELECT EXISTS (
					SELECT 1 FROM links
					WHERE source_id = $1 AND target_id = $2
				) as "exists!"
			"#,
			source_block.id,
			target_block.id
		)
		.fetch_one(&postgres_repo.pool)
		.await
		.expect("Failed to check link existence in PostgreSQL")
		.exists;

		assert!(postgres_link);

		// Act: Delete the link from PostgreSQL, which should also sync to memory.
		postgres_repo
			.delete_content_link(link)
			.await
			.expect("Failed to delete content link");

		// Assert: The link no longer exists in either repository.
		let postgres_link = sqlx::query!(
			r#"
				SELECT EXISTS (
					SELECT 1 FROM links
					WHERE source_id = $1 AND target_id = $2
				) as "exists!"
			"#,
			source_block.id,
			target_block.id
		)
		.fetch_one(&postgres_repo.pool)
		.await
		.expect("Failed to check link existence in PostgreSQL")
		.exists;

		assert!(!postgres_link);
	}
}
