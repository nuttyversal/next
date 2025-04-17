use crate::{
	models::{ContentBlock, ContentLink},
	repository::{
		ContentRepository, ContentRepositoryError, Repository, repository::TransactionExt,
	},
};

pub struct ContentService {
	// The content repository to use for storing and retrieving content.
	repository: ContentRepository,
}

impl ContentService {
	// Create a new content service with the given repository.
	pub fn new(repository: ContentRepository) -> Self {
		ContentService { repository }
	}

	// Save a content block.
	pub async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ContentServiceError> {
		self
			.repository
			.with_transaction(|tx| {
				Box::pin(async move {
					// Save the content block.
					let content_block = self
						.repository
						.upsert_content_block_tx(tx.as_executor(), content_block.clone())
						.await
						.map_err(ContentServiceError::SaveContentBlock)?;

					// Parse tags from the content block.
					let target_tags = content_block.content.parse_target_tags();

					// Resolve [NuttyTag] references.
					let target_ids = self
						.repository
						.resolve_nutty_ids_tx(
							tx.as_executor(),
							target_tags
								.iter()
								.map(|tag| tag.nutty_id())
								.collect::<Vec<_>>(),
						)
						.await;

					// Delete orphaned content links.
					self
						.repository
						.delete_orphaned_content_links_tx(
							tx.as_executor(),
							content_block.nutty_id(),
							&target_ids,
						)
						.await
						.map_err(ContentServiceError::DeleteContentLinks)?;

					// Create new content links.
					let content_links: Vec<ContentLink> = target_ids
						.iter()
						.map(|target_id| ContentLink::now(*content_block.nutty_id(), *target_id))
						.collect();

					// Save the content links.
					self
						.repository
						.upsert_content_links_tx(tx.as_executor(), &content_links)
						.await
						.map_err(ContentServiceError::SaveContentLink)?;

					// Return the saved content block.
					Ok(content_block)
				})
			})
			.await
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ContentServiceError {
	#[error("Failed to save content block: {0}")]
	SaveContentBlock(#[source] ContentRepositoryError),

	#[error("Failed to save content link: {0}")]
	SaveContentLink(#[source] ContentRepositoryError),

	#[error("Failed to delete content links: {0}")]
	DeleteContentLinks(#[source] ContentRepositoryError),

	#[error("Database error: {0}")]
	Database(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::models::{BlockContent, ContentBlock, FractionalIndex};
	use crate::repository::ContentRepository;
	use sqlx::postgres::PgPoolOptions;
	use sqlx::{Pool, Postgres};

	async fn connect_to_test_database() -> Pool<Postgres> {
		PgPoolOptions::new()
			.max_connections(5)
			.connect("postgres://nutty@localhost:5432/nuttyverse")
			.await
			.expect("Failed to connect to test database")
	}

	#[tokio::test]
	async fn test_save_content_block() {
		// Arrange: Create a repository and service.
		let pool = connect_to_test_database().await;
		let repo = ContentRepository::new(pool);
		let service = ContentService::new(repo);

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

		// Act: Save the target block first.
		let saved_target = service
			.save_content_block(target_block.clone())
			.await
			.expect("Failed to save target block");

		// Act: Create a source block with a link to the target.
		let source_with_link = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Paragraph {
				markdown: format!("This links to [[{}]]", saved_target.nutty_id().nid()),
			},
		);

		// Act: Save the source block.
		let saved_source = service
			.save_content_block(source_with_link.clone())
			.await
			.expect("Failed to save source block");

		// Assert: The source block was saved correctly.
		assert_eq!(saved_source.nutty_id(), source_with_link.nutty_id());
		assert_eq!(saved_source.parent_id, source_with_link.parent_id);
		assert!(matches!(
			&saved_source.content,
			BlockContent::Paragraph { markdown } if markdown == &format!("This links to [[{}]]", saved_target.nutty_id().nid())
		));

		// Assert: A link was created between the blocks.
		let links = service
			.repository
			.get_content_links_from(saved_source.nutty_id())
			.await
			.expect("Failed to get links from source block");

		assert_eq!(links.len(), 1);
		assert_eq!(links[0].source_id, *saved_source.nutty_id());
		assert_eq!(links[0].target_id, *saved_target.nutty_id());

		// Act: Update the source block to remove the link.
		let updated_source = ContentBlock::now(
			None,
			FractionalIndex::start(),
			BlockContent::Paragraph {
				markdown: "No more links".to_string(),
			},
		);

		// Act: Save the updated source block.
		let saved_updated = service
			.save_content_block(updated_source.clone())
			.await
			.expect("Failed to save updated source block");

		// Assert: The orphaned link was deleted.
		let links = service
			.repository
			.get_content_links_from(saved_updated.nutty_id())
			.await
			.expect("Failed to get links from updated source block");

		assert!(links.is_empty());
	}
}
