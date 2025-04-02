use crate::errors::ApiError;
use crate::models::ContentBlock;
use crate::repository::traits::ContentRepository;
use async_trait::async_trait;
use sqlx::types::Uuid;

/// A repository for content blocks that uses a PostgreSQL database as its backing store.
pub struct PostgresContentRepository {
	pub pool: sqlx::PgPool,
}

impl PostgresContentRepository {
	pub fn new(pool: sqlx::PgPool) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl ContentRepository for PostgresContentRepository {
	async fn get_content_block(&self, id: Uuid) -> Result<Option<ContentBlock>, ApiError> {
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

		Ok(record.and_then(|r| {
			ContentBlock::deserialize_content(r.content)
				.map_err(ApiError::from)
				.ok()
				.map(|content| ContentBlock {
					id: r.id,
					parent_id: r.parent_id,
					content,
				})
		}))
	}

	async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError> {
		let record = sqlx::query!(
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

		Ok(ContentBlock {
			id: record.id,
			parent_id: record.parent_id,
			content: ContentBlock::deserialize_content(record.content).map_err(ApiError::from)?,
		})
	}

	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError> {
		sqlx::query!(
			r#"
				DELETE FROM blocks
				WHERE id = $1
			"#,
			id
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::BlockContent;
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
}
