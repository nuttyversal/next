use crate::errors::ApiError;
use crate::models::{
	AnyNuttyId, ContentBlock, ContentLink, FractionalIndex, NuttyId, NuttyIdentifier,
};
use crate::repository::traits::ContentRepository;
use async_trait::async_trait;

/// A PostgreSQL repository for content blocks.
pub struct PostgresContentRepository {
	/// The PostgreSQL database pool.
	pool: sqlx::PgPool,
}

impl PostgresContentRepository {
	/// Create a new PostgreSQL content repository.
	pub fn new(pool: sqlx::PgPool) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl ContentRepository for PostgresContentRepository {
	async fn get_content_block(
		&self,
		nutty_id: &AnyNuttyId,
	) -> Result<Option<ContentBlock>, ApiError> {
		// Find the content block.
		let record = sqlx::query!(
			r#"
				SELECT id, parent_id, content, index
				FROM blocks
				WHERE nutty_id = $1
			"#,
			nutty_id.nid()
		)
		.fetch_optional(&self.pool)
		.await?;

		match record {
			// Found the content block!
			Some(record) => {
				let nutty_id = NuttyId::new(record.id);
				let parent_id = record.parent_id.map(NuttyId::new);
				let content = ContentBlock::deserialize_content(record.content)?;
				let index_result = FractionalIndex::new(record.index);
				let index = index_result.map_err(|e| ApiError::InvalidIndex(e.to_string()))?;

				Ok(Some(ContentBlock::new(nutty_id, parent_id, content, index)))
			}

			// It does not exist…
			None => Ok(None),
		}
	}

	async fn upsert_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError> {
		// Upsert the content block.
		let record = sqlx::query!(
			r#"
				INSERT INTO blocks (id, nutty_id, parent_id, content, index)
				VALUES ($1, $2, $3, $4, $5)
				ON CONFLICT (id) DO UPDATE
				SET parent_id = EXCLUDED.parent_id, content = EXCLUDED.content, index = EXCLUDED.index
				RETURNING id, nutty_id, parent_id, content, index
			"#,
			content_block.nutty_id().uuid(),
			content_block.nutty_id().nid(),
			content_block.parent_id.clone().map(|id| id.uuid().clone()),
			content_block.serialize_content()?,
			content_block.index.as_str(),
		)
		.fetch_one(&self.pool)
		.await?;

		// Get the updated content block.
		let nutty_id = NuttyId::new(record.id);
		let parent_id = record.parent_id.map(NuttyId::new);
		let content = ContentBlock::deserialize_content(record.content)?;
		let index_result = FractionalIndex::new(record.index);
		let index = index_result.map_err(|e| ApiError::InvalidIndex(e.to_string()))?;

		Ok(ContentBlock::new(nutty_id, parent_id, content, index))
	}

	async fn delete_content_block(&self, nutty_id: &AnyNuttyId) -> Result<(), ApiError> {
		sqlx::query!(
			r#"
				DELETE FROM blocks
				WHERE nutty_id = $1
			"#,
			nutty_id.nid()
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	async fn get_content_link(
		&self,
		nutty_id: &AnyNuttyId,
	) -> Result<Option<ContentLink>, ApiError> {
		// Find the content link.
		let record = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM links
				WHERE nutty_id = $1
			"#,
			nutty_id.nid()
		)
		.fetch_optional(&self.pool)
		.await?;

		match record {
			// Found the content link!
			Some(record) => Ok(Some(ContentLink::new(
				NuttyId::new(record.id),
				NuttyId::new(record.source_id),
				NuttyId::new(record.target_id),
			))),

			// It does not exist…
			None => Ok(None),
		}
	}

	async fn get_content_links_from(
		&self,
		nutty_id: &NuttyId,
	) -> Result<Vec<ContentLink>, ApiError> {
		let records = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM links
				WHERE source_id = $1
			"#,
			nutty_id.uuid()
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(records
			.iter()
			.map(|record| {
				ContentLink::new(
					NuttyId::new(record.id),
					NuttyId::new(record.source_id),
					NuttyId::new(record.target_id),
				)
			})
			.collect())
	}

	async fn get_content_links_to(&self, nutty_id: &NuttyId) -> Result<Vec<ContentLink>, ApiError> {
		let records = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM links
				WHERE target_id = $1
			"#,
			nutty_id.uuid()
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(records
			.iter()
			.map(|record| {
				ContentLink::new(
					NuttyId::new(record.id),
					NuttyId::new(record.source_id),
					NuttyId::new(record.target_id),
				)
			})
			.collect())
	}

	async fn upsert_content_link(&self, link: ContentLink) -> Result<ContentLink, ApiError> {
		// Insert the content link.
		let record = sqlx::query!(
			r#"
				INSERT INTO links (id, nutty_id, source_id, target_id)
				VALUES ($1, $2, $3, $4)
				ON CONFLICT (id) DO NOTHING
				RETURNING id, nutty_id, source_id, target_id
			"#,
			link.nutty_id.uuid(),
			link.nutty_id.nid(),
			link.source_id.uuid(),
			link.target_id.uuid()
		)
		.fetch_one(&self.pool)
		.await?;

		// Get the updated content link.
		let nutty_id = NuttyId::new(record.id);
		let source_id = NuttyId::new(record.source_id);
		let target_id = NuttyId::new(record.target_id);

		Ok(ContentLink::new(nutty_id, source_id, target_id))
	}

	async fn delete_content_link(&self, link: ContentLink) -> Result<(), ApiError> {
		sqlx::query!(
			r#"
				DELETE FROM links
				WHERE id = $1
			"#,
			link.nutty_id.uuid()
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	async fn is_linked(&self, source_id: &NuttyId, target_id: &NuttyId) -> Result<bool, ApiError> {
		let record = sqlx::query!(
			r#"
				SELECT EXISTS (
					SELECT 1 FROM links
					WHERE source_id = $1 AND target_id = $2
				) as "exists!"
			"#,
			source_id.uuid(),
			target_id.uuid()
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(record.exists)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::repository::tests::{TestRepositoryFactory, test_content_repository};
	use sqlx::PgPool;
	use sqlx::postgres::PgPoolOptions;

	struct PostgresRepositoryFactory {
		pool: PgPool,
	}

	impl PostgresRepositoryFactory {
		async fn new() -> Self {
			let pool = PgPoolOptions::new()
				.max_connections(5)
				.connect("postgres://nutty@localhost:5432/nuttyverse")
				.await
				.expect("Failed to connect to test database");

			Self { pool }
		}
	}

	impl TestRepositoryFactory for PostgresRepositoryFactory {
		type Repository = PostgresContentRepository;

		fn create_repository(&self) -> Self::Repository {
			PostgresContentRepository::new(self.pool.clone())
		}
	}

	#[tokio::test]
	async fn test_postgres_repository() {
		let factory = PostgresRepositoryFactory::new().await;
		test_content_repository(factory).await;
	}
}
