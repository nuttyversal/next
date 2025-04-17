use sqlx::{Executor, Postgres};
use thiserror::Error;

use crate::models::content_block::{ContentBlockBuilderError, ContentBlockError};
use crate::models::fractional_index::FractionalIndexError;
use crate::models::{AnyNuttyId, ContentBlock, ContentLink, FractionalIndex, NuttyId};
use crate::repository::Repository;

/// A repository for content blocks.
/// Objects are stored in PostgreSQL.
pub struct ContentRepository {
	/// The PostgreSQL database pool.
	pool: sqlx::Pool<Postgres>,
}

impl ContentRepository {
	/// Create a new content repository.
	pub fn new(pool: sqlx::Pool<Postgres>) -> Self {
		Self { pool }
	}

	/// Resolve a collection of [AnyNuttyId] into a Vec of [NuttyId].
	pub async fn resolve_nutty_ids_tx<'e, 'i, E, I>(&self, executor: E, ids: I) -> Vec<NuttyId>
	where
		E: Executor<'e, Database = Postgres>,
		I: IntoIterator<Item = &'i AnyNuttyId>,
	{
		// Collect Nutty IDs.
		let nids: Vec<_> = ids.into_iter().map(|id| id.nid()).collect();

		// Query the content blocks.
		let resolved = sqlx::query!(
			r#"
				SELECT id, nutty_id
				FROM content.blocks
				WHERE nutty_id = ANY($1)
			"#,
			&nids,
		)
		.fetch_all(executor)
		.await
		.expect("Failed to resolve Nutty IDs");

		// Map the results.
		resolved
			.into_iter()
			.map(|record| NuttyId::new(record.id))
			.collect()
	}

	/// Resolve a collection of [AnyNuttyId] into a Vec of [NuttyId].
	pub async fn resolve_nutty_ids<'i, I>(&self, ids: I) -> Vec<NuttyId>
	where
		I: IntoIterator<Item = &'i AnyNuttyId>,
	{
		self.resolve_nutty_ids_tx(&self.pool, ids).await
	}

	/// Get a content block by its Nutty ID.
	pub async fn get_content_block_tx<'e, E>(
		&self,
		executor: E,
		nutty_id: &AnyNuttyId,
	) -> Result<Option<ContentBlock>, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		// Find the content block.
		let record = sqlx::query!(
			r#"
				SELECT id, parent_id, f_index, content
				FROM content.blocks
				WHERE nutty_id = $1
			"#,
			nutty_id.nid()
		)
		.fetch_optional(executor)
		.await?;

		match record {
			// Found the content block!
			Some(record) => {
				let nutty_id = NuttyId::new(record.id);
				let parent_id = record.parent_id.map(NuttyId::new);
				let f_index = FractionalIndex::new(record.f_index)?;
				let content = ContentBlock::deserialize_content(record.content)?;

				Ok(Some(
					ContentBlock::builder()
						.nutty_id(nutty_id)
						.parent_id(parent_id)
						.f_index(f_index)
						.content(content)
						.try_build()?,
				))
			}

			// It does not exist…
			None => Ok(None),
		}
	}

	/// Get a content block by its Nutty ID.
	pub async fn get_content_block(
		&self,
		nutty_id: &AnyNuttyId,
	) -> Result<Option<ContentBlock>, ContentRepositoryError> {
		self.get_content_block_tx(&self.pool, nutty_id).await
	}

	/// Upsert a content block.
	pub async fn upsert_content_block_tx<'e, E>(
		&self,
		executor: E,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		// Upsert the content block.
		let record = sqlx::query!(
			r#"
				INSERT INTO content.blocks (id, nutty_id, parent_id, f_index, content)
				VALUES ($1, $2, $3, $4, $5)
				ON CONFLICT (id) DO UPDATE
				SET parent_id = EXCLUDED.parent_id, content = EXCLUDED.content, f_index = EXCLUDED.f_index
				RETURNING id, nutty_id, parent_id, f_index, content
			"#,
			content_block.nutty_id().uuid(),
			content_block.nutty_id().nid(),
			content_block.parent_id.clone().map(|id| id.uuid().clone()),
			content_block.f_index.as_str(),
			content_block.serialize_content()?,
		)
		.fetch_one(executor)
		.await?;

		// Get the updated content block.
		let nutty_id = NuttyId::new(record.id);
		let parent_id = record.parent_id.map(NuttyId::new);
		let f_index = FractionalIndex::new(record.f_index)?;
		let content = ContentBlock::deserialize_content(record.content)?;

		Ok(ContentBlock::builder()
			.nutty_id(nutty_id)
			.parent_id(parent_id)
			.f_index(f_index)
			.content(content)
			.try_build()?)
	}

	/// Upsert a content block.
	pub async fn upsert_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ContentRepositoryError> {
		self
			.upsert_content_block_tx(&self.pool, content_block)
			.await
	}

	/// Delete a block of content by its identifier.
	pub async fn delete_content_block_tx<'e, E>(
		&self,
		executor: E,
		nutty_id: &AnyNuttyId,
	) -> Result<(), ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		sqlx::query!(
			r#"
				DELETE FROM content.blocks
				WHERE nutty_id = $1
			"#,
			nutty_id.nid()
		)
		.execute(executor)
		.await?;

		Ok(())
	}

	/// Delete a block of content by its identifier.
	pub async fn delete_content_block(
		&self,
		nutty_id: &AnyNuttyId,
	) -> Result<(), ContentRepositoryError> {
		self.delete_content_block_tx(&self.pool, nutty_id).await
	}

	/// Get a content link by its identifier.
	pub async fn get_content_link_tx<'e, E>(
		&self,
		executor: E,
		nutty_id: &AnyNuttyId,
	) -> Result<Option<ContentLink>, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		// Find the content link.
		let record = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM content.links
				WHERE nutty_id = $1
			"#,
			nutty_id.nid()
		)
		.fetch_optional(executor)
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

	/// Get a content link by its identifier.
	pub async fn get_content_link(
		&self,
		nutty_id: &AnyNuttyId,
	) -> Result<Option<ContentLink>, ContentRepositoryError> {
		self.get_content_link_tx(&self.pool, nutty_id).await
	}

	/// Get all content links from a content block.
	pub async fn get_content_links_from_tx<'e, E>(
		&self,
		executor: E,
		nutty_id: &NuttyId,
	) -> Result<Vec<ContentLink>, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		let records = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM content.links
				WHERE source_id = $1
			"#,
			nutty_id.uuid()
		)
		.fetch_all(executor)
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

	/// Get all content links from a content block.
	pub async fn get_content_links_from(
		&self,
		nutty_id: &NuttyId,
	) -> Result<Vec<ContentLink>, ContentRepositoryError> {
		self.get_content_links_from_tx(&self.pool, nutty_id).await
	}

	/// Get all content links to a content block.
	pub async fn get_content_links_to_tx<'e, E>(
		&self,
		executor: E,
		nutty_id: &NuttyId,
	) -> Result<Vec<ContentLink>, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		let records = sqlx::query!(
			r#"
				SELECT id, source_id, target_id
				FROM content.links
				WHERE target_id = $1
			"#,
			nutty_id.uuid()
		)
		.fetch_all(executor)
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

	/// Get all content links to a content block.
	pub async fn get_content_links_to(
		&self,
		nutty_id: &NuttyId,
	) -> Result<Vec<ContentLink>, ContentRepositoryError> {
		self.get_content_links_to_tx(&self.pool, nutty_id).await
	}

	/// Upsert a content link between two content blocks.
	pub async fn upsert_content_link_tx<'e, E>(
		&self,
		executor: E,
		link: ContentLink,
	) -> Result<ContentLink, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		// Insert the content link.
		let record = sqlx::query!(
			r#"
				INSERT INTO content.links (id, nutty_id, source_id, target_id)
				VALUES ($1, $2, $3, $4)
				ON CONFLICT (id) DO NOTHING
				RETURNING id, nutty_id, source_id, target_id
			"#,
			link.nutty_id.uuid(),
			link.nutty_id.nid(),
			link.source_id.uuid(),
			link.target_id.uuid()
		)
		.fetch_one(executor)
		.await?;

		// Get the updated content link.
		let nutty_id = NuttyId::new(record.id);
		let source_id = NuttyId::new(record.source_id);
		let target_id = NuttyId::new(record.target_id);

		Ok(ContentLink::new(nutty_id, source_id, target_id))
	}

	/// Upsert a content link between two content blocks.
	pub async fn upsert_content_link(
		&self,
		link: ContentLink,
	) -> Result<ContentLink, ContentRepositoryError> {
		self.upsert_content_link_tx(&self.pool, link).await
	}

	/// Upsert multiple content links.
	pub async fn upsert_content_links_tx<'e, E>(
		&self,
		executor: E,
		links: &[ContentLink],
	) -> Result<Vec<ContentLink>, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		// Prepare the data for the bulk insert.
		let ids = links
			.iter()
			.map(|link| *link.nutty_id.uuid())
			.collect::<Vec<_>>();
		let nids = links
			.iter()
			.map(|link| link.nutty_id.nid())
			.collect::<Vec<_>>();
		let source_ids = links
			.iter()
			.map(|link| *link.source_id.uuid())
			.collect::<Vec<_>>();
		let target_ids = links
			.iter()
			.map(|link| *link.target_id.uuid())
			.collect::<Vec<_>>();

		// Execute the bulk insert.
		let records = sqlx::query!(
			r#"
				INSERT INTO content.links (id, nutty_id, source_id, target_id)
				SELECT * FROM UNNEST($1::uuid[], $2::text[], $3::uuid[], $4::uuid[])
				ON CONFLICT (source_id, target_id) DO NOTHING
				RETURNING id, nutty_id, source_id, target_id
			"#,
			&ids,
			&nids,
			&source_ids,
			&target_ids,
		)
		.fetch_all(executor)
		.await?;

		// Map the results.
		Ok(records
			.into_iter()
			.map(|record| {
				ContentLink::new(
					NuttyId::new(record.id),
					NuttyId::new(record.source_id),
					NuttyId::new(record.target_id),
				)
			})
			.collect())
	}

	/// Upsert multiple content links.
	pub async fn upsert_content_links(
		&self,
		links: &[ContentLink],
	) -> Result<Vec<ContentLink>, ContentRepositoryError> {
		self.upsert_content_links_tx(&self.pool, links).await
	}

	/// Delete a content link between two content blocks.
	pub async fn delete_content_link_tx<'e, E>(
		&self,
		executor: E,
		link: ContentLink,
	) -> Result<(), ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		sqlx::query!(
			r#"
				DELETE FROM content.links
				WHERE id = $1
			"#,
			link.nutty_id.uuid()
		)
		.execute(executor)
		.await?;

		Ok(())
	}

	/// Delete a content link between two content blocks.
	pub async fn delete_content_link(
		&self,
		link: ContentLink,
	) -> Result<(), ContentRepositoryError> {
		self.delete_content_link_tx(&self.pool, link).await
	}

	/// Delete content links orphaned from the source block.
	pub async fn delete_orphaned_content_links_tx<'e, E>(
		&self,
		executor: E,
		source_id: &NuttyId,
		target_ids: &[NuttyId],
	) -> Result<(), ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		sqlx::query!(
			r#"
				DELETE FROM content.links
				WHERE source_id = $1
				AND target_id <> ANY($2)
			"#,
			source_id.uuid(),
			&target_ids
				.iter()
				.map(|id| id.uuid().clone())
				.collect::<Vec<_>>()
		)
		.execute(executor)
		.await?;

		Ok(())
	}

	/// Delete content links orphaned from the source block.
	pub async fn delete_orphaned_content_links(
		&self,
		source_id: &NuttyId,
		target_ids: &[NuttyId],
	) -> Result<(), ContentRepositoryError> {
		self
			.delete_orphaned_content_links_tx(&self.pool, source_id, target_ids)
			.await
	}

	/// Check if two content blocks are linked.
	pub async fn is_linked_tx<'e, E>(
		&self,
		executor: E,
		source_id: &NuttyId,
		target_id: &NuttyId,
	) -> Result<bool, ContentRepositoryError>
	where
		E: Executor<'e, Database = Postgres>,
	{
		let record = sqlx::query!(
			r#"
				SELECT EXISTS (
					SELECT 1 FROM content.links
					WHERE source_id = $1 AND target_id = $2
				) AS "exists!"
			"#,
			source_id.uuid(),
			target_id.uuid()
		)
		.fetch_one(executor)
		.await?;

		Ok(record.exists)
	}

	/// Check if two content blocks are linked.
	pub async fn is_linked(
		&self,
		source_id: &NuttyId,
		target_id: &NuttyId,
	) -> Result<bool, ContentRepositoryError> {
		self.is_linked_tx(&self.pool, source_id, target_id).await
	}
}

impl Repository for ContentRepository {
	fn pool(&self) -> &sqlx::Pool<Postgres> {
		&self.pool
	}
}

#[derive(Debug, Error)]
pub enum ContentRepositoryError {
	#[error("Unable to query content blocks: {0}")]
	QueryFailed(#[from] sqlx::error::Error),

	#[error("Invalid content block builder state: {0}")]
	InvalidContentBlockBuilder(#[from] ContentBlockBuilderError),

	#[error("Invalid content block: {0}")]
	InvalidContentBlock(#[from] ContentBlockError),

	#[error("Invalid index: {0}")]
	InvalidFractionalIndex(#[from] FractionalIndexError),
}

#[cfg(test)]
mod tests {
	use sqlx::postgres::PgPoolOptions;
	use sqlx::{Pool, Postgres};

	use crate::models::{
		AnyNuttyId, BlockContent, ContentBlock, ContentLink, DissociatedNuttyId, FractionalIndex,
		NuttyId,
	};
	use crate::repository::ContentRepository;

	async fn connect_to_test_database() -> Pool<Postgres> {
		PgPoolOptions::new()
			.max_connections(5)
			.connect("postgres://nutty@localhost:5432/nuttyverse")
			.await
			.expect("Failed to connect to test database")
	}

	#[tokio::test]
	async fn test_content_block_operations() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool);

		// Arrange: Create a test content block.
		let test_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page".to_string(),
			},
		);

		// Act: Save the test content block.
		let saved_block = repo
			.upsert_content_block(test_block.clone())
			.await
			.expect("Failed to save content block");

		// Assert: The saved content block matches the original.
		assert_eq!(saved_block.nutty_id(), test_block.nutty_id());
		assert_eq!(saved_block.parent_id, test_block.parent_id);
		assert!(matches!(&saved_block.content, BlockContent::Page { title } if title == "Test Page"));

		// Act: Query the content block.
		let retrieved = repo
			.get_content_block(&(*saved_block.nutty_id()).into())
			.await
			.expect("Failed to get content block")
			.expect("Content block not found");

		// Assert: The retrieved content block matches the original.
		assert_eq!(retrieved.nutty_id(), test_block.nutty_id());
		assert_eq!(retrieved.parent_id, test_block.parent_id);
		assert!(matches!(retrieved.content, BlockContent::Page { title } if title == "Test Page"));

		// Act: Update the content block.
		let updated_block = ContentBlock::builder()
			.nutty_id(*test_block.nutty_id())
			.parent_id(test_block.parent_id)
			.f_index(test_block.f_index.clone())
			.content(BlockContent::Page {
				title: "Updated Page".to_string(),
			})
			.try_build()
			.unwrap();

		let updated = repo
			.upsert_content_block(updated_block)
			.await
			.expect("Failed to update content block");

		// Assert: The content block was updated.
		assert_eq!(updated.nutty_id(), test_block.nutty_id());
		assert_eq!(updated.parent_id, test_block.parent_id);
		assert!(matches!(updated.content, BlockContent::Page { title } if title == "Updated Page"));

		// Act: Delete the content block.
		repo
			.delete_content_block(&(*test_block.nutty_id()).into())
			.await
			.expect("Failed to delete content block");

		// Assert: The content block no longer exists.
		let retrieved = repo
			.get_content_block(&(*test_block.nutty_id()).into())
			.await
			.unwrap();
		assert!(retrieved.is_none());
	}

	#[tokio::test]
	async fn test_content_link_operations() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool);

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block = ContentBlock::now(
			None,
			FractionalIndex::between(&source_block.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Target Page".to_string(),
			},
		);

		// Act: Save the content blocks.
		repo
			.upsert_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		repo
			.upsert_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a content link between the blocks.
		let link = ContentLink::now(*source_block.nutty_id(), *target_block.nutty_id());

		repo
			.upsert_content_link(link.clone())
			.await
			.expect("Failed to create content link");

		// Assert: The blocks are linked.
		assert!(
			repo
				.is_linked(
					&source_block.nutty_id().clone(),
					&target_block.nutty_id().clone()
				)
				.await
				.expect("Failed to check link")
		);

		// Act: Get the content link by ID.
		let retrieved_link = repo
			.get_content_link(&link.nutty_id.into())
			.await
			.expect("Failed to get content link")
			.expect("Content link not found");

		// Assert: The retrieved link matches the original.
		assert_eq!(retrieved_link.nutty_id, link.nutty_id);
		assert_eq!(retrieved_link.source_id, *source_block.nutty_id());
		assert_eq!(retrieved_link.target_id, *target_block.nutty_id());

		// Act: Get all links from the source block.
		let links_from = repo
			.get_content_links_from(source_block.nutty_id())
			.await
			.expect("Failed to get links from source block");

		// Assert: The links from source block match the original.
		assert_eq!(links_from.len(), 1);
		assert_eq!(links_from[0].nutty_id, link.nutty_id);
		assert_eq!(links_from[0].source_id, *source_block.nutty_id());
		assert_eq!(links_from[0].target_id, *target_block.nutty_id());

		// Act: Get all links to the target block.
		let links_to = repo
			.get_content_links_to(target_block.nutty_id())
			.await
			.expect("Failed to get links to target block");

		// Assert: The links to target block match the original.
		assert_eq!(links_to.len(), 1);
		assert_eq!(links_to[0].nutty_id, link.nutty_id);
		assert_eq!(links_to[0].source_id, *source_block.nutty_id());
		assert_eq!(links_to[0].target_id, *target_block.nutty_id());

		// Act: Try to get a non-existent link.
		let non_existent_link = repo
			.get_content_link(&AnyNuttyId::Associated(NuttyId::now()))
			.await
			.expect("Failed to check non-existent link");

		// Assert: No link is found.
		assert!(non_existent_link.is_none());

		// Act: Try to get links from a non-existent source.
		let no_links_from = repo
			.get_content_links_from(&NuttyId::now())
			.await
			.expect("Failed to get links from non-existent source");

		// Assert: No links are found.
		assert!(no_links_from.is_empty());

		// Act: Try to get links to a non-existent target.
		let no_links_to = repo
			.get_content_links_to(&NuttyId::now())
			.await
			.expect("Failed to get links to non-existent target");

		// Assert: No links are found.
		assert!(no_links_to.is_empty());

		// Act: Delete the link.
		repo
			.delete_content_link(link)
			.await
			.expect("Failed to delete content link");

		// Assert: The blocks are no longer linked.
		assert!(
			!repo
				.is_linked(source_block.nutty_id(), target_block.nutty_id())
				.await
				.expect("Failed to check link")
		);
	}

	#[tokio::test]
	async fn test_resolve_nutty_ids() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool);

		// Arrange: Create test content blocks.
		let block1 = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Test Page 1".to_string(),
			},
		);

		let block2 = ContentBlock::now(
			None,
			FractionalIndex::between(&block1.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Test Page 2".to_string(),
			},
		);

		// Act: Save the content blocks.
		repo
			.upsert_content_block(block1.clone())
			.await
			.expect("Failed to save block1");

		repo
			.upsert_content_block(block2.clone())
			.await
			.expect("Failed to save block2");

		// Act: Create a mix of associated and dissociated IDs.
		let ids = [
			AnyNuttyId::Associated(*block1.nutty_id()),
			AnyNuttyId::Dissociated(
				DissociatedNuttyId::new(&block2.nutty_id().nid())
					.expect("Failed to create dissociated ID"),
			),
			AnyNuttyId::Associated(NuttyId::now()),
			AnyNuttyId::Dissociated(
				DissociatedNuttyId::new("1111111").expect("Failed to create dissociated ID"),
			),
		];

		// Act: Resolve the IDs.
		let resolved = repo.resolve_nutty_ids(ids.iter()).await;

		// Assert: Only the IDs that exist in the database are returned.
		assert_eq!(resolved.len(), 2);
		assert!(resolved.contains(block1.nutty_id()));
		assert!(resolved.contains(block2.nutty_id()));
	}

	#[tokio::test]
	async fn test_delete_orphaned_content_links() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool);

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block1 = ContentBlock::now(
			None,
			FractionalIndex::between(&source_block.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Target Page 1".to_string(),
			},
		);

		let target_block2 = ContentBlock::now(
			None,
			FractionalIndex::between(&target_block1.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Target Page 2".to_string(),
			},
		);

		// Act: Save the content blocks.
		repo
			.upsert_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		repo
			.upsert_content_block(target_block1.clone())
			.await
			.expect("Failed to save target block 1");

		repo
			.upsert_content_block(target_block2.clone())
			.await
			.expect("Failed to save target block 2");

		// Act: Create content links between the blocks.
		let link1 = ContentLink::now(*source_block.nutty_id(), *target_block1.nutty_id());
		let link2 = ContentLink::now(*source_block.nutty_id(), *target_block2.nutty_id());

		repo
			.upsert_content_link(link1.clone())
			.await
			.expect("Failed to create link 1");

		repo
			.upsert_content_link(link2.clone())
			.await
			.expect("Failed to create link 2");

		// Assert: Both links exist initially.
		assert!(
			repo
				.is_linked(source_block.nutty_id(), target_block1.nutty_id())
				.await
				.expect("Failed to check link 1")
		);
		assert!(
			repo
				.is_linked(source_block.nutty_id(), target_block2.nutty_id())
				.await
				.expect("Failed to check link 2")
		);

		// Act: Delete orphaned links, keeping only link1.
		repo
			.delete_orphaned_content_links(source_block.nutty_id(), &[*target_block1.nutty_id()])
			.await
			.expect("Failed to delete orphaned links");

		// Assert: Only link1 remains.
		assert!(
			repo
				.is_linked(source_block.nutty_id(), target_block1.nutty_id())
				.await
				.expect("Failed to check link 1")
		);
		assert!(
			!repo
				.is_linked(source_block.nutty_id(), target_block2.nutty_id())
				.await
				.expect("Failed to check link 2")
		);
	}

	#[tokio::test]
	async fn test_upsert_content_links() {
		// Arrange: Create a repository.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool);

		// Arrange: Create test content blocks.
		let source_block = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Page {
				title: "Source Page".to_string(),
			},
		);

		let target_block1 = ContentBlock::now(
			None,
			FractionalIndex::between(&source_block.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Target Page 1".to_string(),
			},
		);

		let target_block2 = ContentBlock::now(
			None,
			FractionalIndex::between(&target_block1.f_index, &FractionalIndex::end()).unwrap(),
			BlockContent::Page {
				title: "Target Page 2".to_string(),
			},
		);

		// Act: Save the content blocks.
		repo
			.upsert_content_block(source_block.clone())
			.await
			.expect("Failed to save source block");

		repo
			.upsert_content_block(target_block1.clone())
			.await
			.expect("Failed to save target block 1");

		repo
			.upsert_content_block(target_block2.clone())
			.await
			.expect("Failed to save target block 2");

		// Act: Create multiple content links.
		let links = vec![
			ContentLink::now(*source_block.nutty_id(), *target_block1.nutty_id()),
			ContentLink::now(*source_block.nutty_id(), *target_block2.nutty_id()),
		];

		// Act: Save the links in bulk.
		let saved_links = repo
			.upsert_content_links(&links)
			.await
			.expect("Failed to save content links");

		// Assert: The correct number of links were saved.
		assert_eq!(saved_links.len(), 2);

		// Assert: Each link was saved correctly.
		for saved_link in saved_links {
			assert_eq!(saved_link.source_id, *source_block.nutty_id());
			assert!(
				saved_link.target_id == *target_block1.nutty_id()
					|| saved_link.target_id == *target_block2.nutty_id()
			);
		}

		// Act: Try to save duplicate links (should be ignored due to unique constraint).
		let duplicate_links = vec![
			ContentLink::now(*source_block.nutty_id(), *target_block1.nutty_id()),
			ContentLink::now(*source_block.nutty_id(), *target_block2.nutty_id()),
		];

		let saved_duplicates = repo
			.upsert_content_links(&duplicate_links)
			.await
			.expect("Failed to save duplicate links");

		// Assert: No new links were created.
		assert_eq!(saved_duplicates.len(), 0);

		// Assert: The original links still exist.
		let links_from = repo
			.get_content_links_from(source_block.nutty_id())
			.await
			.expect("Failed to get links from source block");

		assert_eq!(links_from.len(), 2);
	}
}
