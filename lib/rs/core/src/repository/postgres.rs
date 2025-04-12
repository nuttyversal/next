use crate::errors::ApiError;
use crate::models::{ContentBlock, ContentLink, FractionalIndex, NuttyId};
use crate::repository::traits::ContentRepository;
use async_trait::async_trait;
use sqlx::{Row, types::Uuid};
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
		let record = sqlx::query(
			r#"
				SELECT id, nutty_id, parent_id, content, index
				FROM blocks
				WHERE id = $1
			"#,
		)
		.bind(id)
		.fetch_optional(&self.pool)
		.await?;

		match (record, &*self.linked_repository.read().await) {
			// Found the content block in the database!
			(Some(record), _) => {
				let nutty_id = NuttyId::new(record.get("nutty_id"));
				let content = ContentBlock::deserialize_content(record.get("content"))?;
				let index = FractionalIndex::new(record.get("index"))
					.map_err(|e| ApiError::InvalidIndex(e.to_string()))?;

				Ok(Some(ContentBlock {
					id: record.get("id"),
					nutty_id,
					parent_id: record.get("parent_id"),
					content,
					index,
				}))
			}

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
		sqlx::query(
			r#"
				INSERT INTO blocks (id, nutty_id, parent_id, content, index)
				VALUES ($1, $2, $3, $4, $5)
				ON CONFLICT (id) DO UPDATE
				SET parent_id = EXCLUDED.parent_id, content = EXCLUDED.content, index = EXCLUDED.index
				RETURNING id, nutty_id, parent_id, content, index
			"#,
		)
		.bind(content_block.id)
		.bind(content_block.nutty_id.as_str())
		.bind(content_block.parent_id)
		.bind(content_block.serialize_content()?)
		.bind(content_block.index.as_str())
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

	async fn get_content_link(&self, id: Uuid) -> Result<Option<ContentLink>, ApiError> {
		// Try to find the content link in the database.
		let record = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM links
				WHERE id = $1
			"#,
			id
		)
		.fetch_optional(&self.pool)
		.await?;

		match (record, &*self.linked_repository.read().await) {
			// Found the content link in the database!
			(Some(record), _) => Ok(Some(ContentLink {
				id: record.id,
				source_id: record.source_id,
				target_id: record.target_id,
			})),

			// Try to find the content link in the linked repository.
			(_, Some(linked_repository)) => linked_repository.get_content_link(id).await,

			// Cannot find the content link anywhere.
			(_, None) => Ok(None),
		}
	}

	async fn get_content_links_from(&self, id: Uuid) -> Result<Vec<ContentLink>, ApiError> {
		// Get all links from this source block from the database.
		let mut links = std::collections::HashMap::new();

		let records = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM links
				WHERE source_id = $1
			"#,
			id
		)
		.fetch_all(&self.pool)
		.await?;

		for record in records {
			let link = ContentLink {
				id: record.id,
				source_id: record.source_id,
				target_id: record.target_id,
			};

			links.insert(link.id, link);
		}

		// Try to get additional links from the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => {
				let linked_links = linked_repository.get_content_links_from(id).await?;

				for link in linked_links {
					links.insert(link.id, link);
				}

				Ok(links.into_values().collect())
			}

			None => Ok(links.into_values().collect()),
		}
	}

	async fn get_content_links_to(&self, id: Uuid) -> Result<Vec<ContentLink>, ApiError> {
		// Get all links to this target block from the database.
		let mut links = std::collections::HashMap::new();

		let records = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM links
				WHERE target_id = $1
			"#,
			id
		)
		.fetch_all(&self.pool)
		.await?;

		for record in records {
			let link = ContentLink {
				id: record.id,
				source_id: record.source_id,
				target_id: record.target_id,
			};

			links.insert(link.id, link);
		}

		// Try to get additional links from the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => {
				let linked_links = linked_repository.get_content_links_to(id).await?;

				for link in linked_links {
					links.insert(link.id, link);
				}

				Ok(links.into_values().collect())
			}

			None => Ok(links.into_values().collect()),
		}
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

	async fn are_blocks_linked(&self, source_id: Uuid, target_id: Uuid) -> Result<bool, ApiError> {
		let record = sqlx::query!(
			r#"
				SELECT EXISTS (
					SELECT 1 FROM links
					WHERE source_id = $1 AND target_id = $2
				) as "exists!"
			"#,
			source_id,
			target_id
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(record.exists)
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
