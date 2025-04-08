use crate::index::FractionalIndex;
use crate::models::{BlockContent, ContentBlock, ContentLink};
use crate::repository::traits::ContentRepository;
use std::sync::Arc;
use uuid::Uuid;

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
	test_linked_repository_operations(&factory).await;
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
		.save_content_block(test_block.clone())
		.await
		.expect("Failed to save content block");

	// Assert: The saved content block matches the original.
	assert_eq!(saved.id, test_block.id);
	assert_eq!(saved.parent_id, test_block.parent_id);
	assert!(matches!(saved.content, BlockContent::Page { title } if title == "Test Page"));

	// Act: Query the content block.
	let retrieved = repo
		.get_content_block(test_block.id)
		.await
		.expect("Failed to get content block")
		.expect("Content block not found");

	// Assert: The retrieved content block matches the original.
	assert_eq!(retrieved.id, test_block.id);
	assert_eq!(retrieved.parent_id, test_block.parent_id);
	assert!(matches!(retrieved.content, BlockContent::Page { title } if title == "Test Page"));

	// Act: Update the content block.
	let updated_block = ContentBlock::new(
		test_block.id,
		test_block.parent_id,
		BlockContent::Page {
			title: "Updated Page".to_string(),
		},
		test_block.index,
	);

	let updated = repo
		.save_content_block(updated_block)
		.await
		.expect("Failed to update content block");

	// Assert: The content block was updated.
	assert_eq!(updated.id, test_block.id);
	assert_eq!(updated.parent_id, test_block.parent_id);
	assert!(matches!(updated.content, BlockContent::Page { title } if title == "Updated Page"));

	// Act: Delete the content block.
	repo
		.delete_content_block(test_block.id)
		.await
		.expect("Failed to delete content block");

	// Assert: The content block no longer exists.
	let retrieved = repo.get_content_block(test_block.id).await.unwrap();
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
		.save_content_block(source_block.clone())
		.await
		.expect("Failed to save source block");

	repo
		.save_content_block(target_block.clone())
		.await
		.expect("Failed to save target block");

	// Act: Create a content link between the blocks.
	let link = ContentLink::now(source_block.id, target_block.id);

	repo
		.save_content_link(link)
		.await
		.expect("Failed to create content link");

	// Assert: The blocks are linked.
	assert!(
		repo
			.are_blocks_linked(source_block.id, target_block.id)
			.await
			.expect("Failed to check link")
	);

	// Act: Get the content link by ID.
	let retrieved_link = repo
		.get_content_link(link.id)
		.await
		.expect("Failed to get content link")
		.expect("Content link not found");

	// Assert: The retrieved link matches the original.
	assert_eq!(retrieved_link.id, link.id);
	assert_eq!(retrieved_link.source_id, source_block.id);
	assert_eq!(retrieved_link.target_id, target_block.id);

	// Act: Get all links from the source block.
	let links_from = repo
		.get_content_links_from(source_block.id)
		.await
		.expect("Failed to get links from source block");

	// Assert: The links from source block match the original.
	assert_eq!(links_from.len(), 1);
	assert_eq!(links_from[0].id, link.id);
	assert_eq!(links_from[0].source_id, source_block.id);
	assert_eq!(links_from[0].target_id, target_block.id);

	// Act: Get all links to the target block.
	let links_to = repo
		.get_content_links_to(target_block.id)
		.await
		.expect("Failed to get links to target block");

	// Assert: The links to target block match the original.
	assert_eq!(links_to.len(), 1);
	assert_eq!(links_to[0].id, link.id);
	assert_eq!(links_to[0].source_id, source_block.id);
	assert_eq!(links_to[0].target_id, target_block.id);

	// Act: Try to get a non-existent link.
	let non_existent_link = repo
		.get_content_link(Uuid::now_v7())
		.await
		.expect("Failed to check non-existent link");

	// Assert: No link is found.
	assert!(non_existent_link.is_none());

	// Act: Try to get links from a non-existent source.
	let no_links_from = repo
		.get_content_links_from(Uuid::now_v7())
		.await
		.expect("Failed to get links from non-existent source");

	// Assert: No links are found.
	assert!(no_links_from.is_empty());

	// Act: Try to get links to a non-existent target.
	let no_links_to = repo
		.get_content_links_to(Uuid::now_v7())
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
			.are_blocks_linked(source_block.id, target_block.id)
			.await
			.expect("Failed to check link")
	);
}

async fn test_linked_repository_operations<F>(factory: &F)
where
	F: TestRepositoryFactory,
{
	// Arrange: Create repositories.
	let mut primary_repo = factory.create_repository();
	let secondary_repo = factory.create_repository();

	// Act: Link the repositories.
	primary_repo
		.link_repository(Arc::new(secondary_repo))
		.await
		.expect("Failed to link repositories");

	// Arrange: Create a test content block.
	let test_block = ContentBlock::now(
		None,
		BlockContent::Page {
			title: "Linked Test Page".to_string(),
		},
		FractionalIndex::start(),
	);

	// Act: Save to primary repository, which should also sync to secondary.
	primary_repo
		.save_content_block(test_block.clone())
		.await
		.expect("Failed to save content block");

	// Assert: Block exists in both repositories.
	let primary_block = primary_repo
		.get_content_block(test_block.id)
		.await
		.expect("Failed to get from primary repository")
		.expect("Block not found in primary repository");

	let secondary_block = primary_repo
		.get_content_block(test_block.id)
		.await
		.expect("Failed to get from secondary repository")
		.expect("Block not found in secondary repository");

	assert_eq!(primary_block.id, secondary_block.id);
	assert_eq!(primary_block.parent_id, secondary_block.parent_id);
	assert!(matches!(
		 primary_block.content,
		 BlockContent::Page { title } if title == "Linked Test Page"
	));

	// Act: Update in primary repository.
	let updated_block = ContentBlock::new(
		test_block.id,
		test_block.parent_id,
		BlockContent::Page {
			title: "Updated Linked Page".to_string(),
		},
		test_block.index,
	);

	primary_repo
		.save_content_block(updated_block)
		.await
		.expect("Failed to update in primary repository");

	// Assert: Update synced to secondary repository.
	let secondary_block = primary_repo
		.get_content_block(test_block.id)
		.await
		.expect("Failed to get from secondary repository")
		.expect("Block not found in secondary repository");

	assert!(matches!(
		 secondary_block.content,
		 BlockContent::Page { title } if title == "Updated Linked Page"
	));

	// Act: Delete from primary repository.
	primary_repo
		.delete_content_block(test_block.id)
		.await
		.expect("Failed to delete from primary repository");

	// Assert: Deletion synced to secondary repository.
	let secondary_block = primary_repo
		.get_content_block(test_block.id)
		.await
		.expect("Failed to get from secondary repository");

	assert!(secondary_block.is_none());

	// Arrange: Create test content blocks for link testing.
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
	primary_repo
		.save_content_block(source_block.clone())
		.await
		.expect("Failed to save source block");

	primary_repo
		.save_content_block(target_block.clone())
		.await
		.expect("Failed to save target block");

	// Act: Create a content link in primary repository.
	let link = ContentLink::now(source_block.id, target_block.id);

	primary_repo
		.save_content_link(link)
		.await
		.expect("Failed to create content link");

	// Assert: The blocks are linked in both repositories.
	assert!(
		primary_repo
			.are_blocks_linked(source_block.id, target_block.id)
			.await
			.expect("Failed to check link in primary repository")
	);

	assert!(
		primary_repo
			.are_blocks_linked(source_block.id, target_block.id)
			.await
			.expect("Failed to check link in secondary repository")
	);

	// Act: Get the link from both repositories.
	let primary_link = primary_repo
		.get_content_link(link.id)
		.await
		.expect("Failed to get link from primary repository")
		.expect("Link not found in primary repository");

	let secondary_link = primary_repo
		.get_content_link(link.id)
		.await
		.expect("Failed to get link from secondary repository")
		.expect("Link not found in secondary repository");

	// Assert: The links match in both repositories.
	assert_eq!(primary_link.id, secondary_link.id);
	assert_eq!(primary_link.source_id, secondary_link.source_id);
	assert_eq!(primary_link.target_id, secondary_link.target_id);

	// Act: Get all links from source block in both repositories.
	let primary_links_from = primary_repo
		.get_content_links_from(source_block.id)
		.await
		.expect("Failed to get links from primary repository");

	let secondary_links_from = primary_repo
		.get_content_links_from(source_block.id)
		.await
		.expect("Failed to get links from secondary repository");

	// Assert: The links from source block match in both repositories.
	assert_eq!(primary_links_from.len(), secondary_links_from.len());
	assert_eq!(primary_links_from[0].id, secondary_links_from[0].id);

	// Act: Get all links to target block in both repositories.
	let primary_links_to = primary_repo
		.get_content_links_to(target_block.id)
		.await
		.expect("Failed to get links to primary repository");

	let secondary_links_to = primary_repo
		.get_content_links_to(target_block.id)
		.await
		.expect("Failed to get links to secondary repository");

	// Assert: The links to target block match in both repositories.
	assert_eq!(primary_links_to.len(), secondary_links_to.len());
	assert_eq!(primary_links_to[0].id, secondary_links_to[0].id);

	// Act: Delete the link from primary repository.
	primary_repo
		.delete_content_link(link)
		.await
		.expect("Failed to delete content link");

	// Assert: The blocks are no longer linked in either repository.
	assert!(
		!primary_repo
			.are_blocks_linked(source_block.id, target_block.id)
			.await
			.expect("Failed to check link in primary repository")
	);

	assert!(
		!primary_repo
			.are_blocks_linked(source_block.id, target_block.id)
			.await
			.expect("Failed to check link in secondary repository")
	);
}
