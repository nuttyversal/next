use crate::models::{
	AnyNuttyId, BlockContent, ContentBlock, ContentLink, FractionalIndex, NuttyId,
};
use crate::repository::traits::ContentRepository;

/// A trait for creating test repositories.
pub trait TestRepositoryFactory {
	/// The type of repository that the factory can create.
	type Repository: ContentRepository + 'static;

	/// Create a new repository.
	fn create_repository(&self) -> Self::Repository;
}

/// A common test suite for [ContentRepository] implementations.
pub async fn test_content_repository<F>(factory: F)
where
	F: TestRepositoryFactory,
{
	test_content_block_operations(&factory).await;
	test_content_link_operations(&factory).await;
}

async fn test_content_block_operations<F>(factory: &F)
where
	F: TestRepositoryFactory,
{
	// Arrange: Create a repository.
	let repo = factory.create_repository();

	// Arrange: Create a test content block.
	let test_block = ContentBlock::now(
		None,
		BlockContent::Page {
			title: "Test Page".to_string(),
		},
		FractionalIndex::start(),
	);

	// Act: Save the test content block.
	let saved = repo
		.upsert_content_block(test_block.clone())
		.await
		.expect("Failed to save content block");

	// Assert: The saved content block matches the original.
	assert_eq!(saved.nutty_id(), test_block.nutty_id());
	assert_eq!(saved.parent_id, test_block.parent_id);
	assert!(matches!(saved.content, BlockContent::Page { title } if title == "Test Page"));

	// Act: Query the content block.
	let retrieved = repo
		.get_content_block(&(*test_block.nutty_id()).into())
		.await
		.expect("Failed to get content block")
		.expect("Content block not found");

	// Assert: The retrieved content block matches the original.
	assert_eq!(retrieved.nutty_id(), test_block.nutty_id());
	assert_eq!(retrieved.parent_id, test_block.parent_id);
	assert!(matches!(retrieved.content, BlockContent::Page { title } if title == "Test Page"));

	// Act: Update the content block.
	let updated_block = ContentBlock::new(
		*test_block.nutty_id(),
		test_block.parent_id,
		BlockContent::Page {
			title: "Updated Page".to_string(),
		},
		test_block.index.clone(),
	);

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

async fn test_content_link_operations<F>(factory: &F)
where
	F: TestRepositoryFactory,
{
	// Arrange: Create a repository.
	let repo = factory.create_repository();

	// Arrange: Create test content blocks.
	let source_block = ContentBlock::now(
		None,
		BlockContent::Page {
			title: "Source Page".to_string(),
		},
		FractionalIndex::start(),
	);

	let target_block = ContentBlock::now(
		None,
		BlockContent::Page {
			title: "Target Page".to_string(),
		},
		FractionalIndex::between(&source_block.index, &FractionalIndex::end()).unwrap(),
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
