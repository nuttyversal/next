use crate::errors::ApiError;
use crate::models::{ContentBlock, ContentLink};
use crate::repository::traits::ContentRepository;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A repository that stores content blocks in memory.
pub struct MemoryContentRepository {
	/// The internal state of the repository, protected by a single RwLock.
	state: RwLock<RepositoryState>,

	/// A linked repository — used to connect to another repository to sync
	/// content blocks during fetching and saving operations.
	linked_repository: RwLock<Option<Arc<dyn ContentRepository>>>,
}

/// The internal state of the repository.
struct RepositoryState {
	/// The content blocks in this repository.
	blocks: HashMap<Uuid, ContentBlock>,

	/// The content links in this repository.
	links: HashMap<Uuid, ContentLink>,

	/// Maps each block ID to the content links where it is the source.
	/// This is used to quickly look up all the links for a given block.
	source_block_links: HashMap<Uuid, HashSet<Uuid>>,

	/// Maps each block ID to the content links where it is the target.
	/// This is used to quickly look up all the links for a given block.
	target_block_links: HashMap<Uuid, HashSet<Uuid>>,
}

impl MemoryContentRepository {
	/// Create a new memory repository.
	pub fn new() -> Self {
		Self {
			state: RwLock::new(RepositoryState {
				blocks: HashMap::new(),
				links: HashMap::new(),
				source_block_links: HashMap::new(),
				target_block_links: HashMap::new(),
			}),

			linked_repository: RwLock::new(None),
		}
	}
}

impl Default for MemoryContentRepository {
	fn default() -> Self {
		Self::new()
	}
}

#[async_trait]
impl ContentRepository for MemoryContentRepository {
	async fn get_content_block(&self, id: Uuid) -> Result<Option<ContentBlock>, ApiError> {
		// Try to find the content block in memory.
		let state = self.state.read().await;

		if let Some(block) = state.blocks.get(&id) {
			return Ok(Some(block.clone()));
		}

		// Try to find the content block in the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.get_content_block(id).await,
			None => Ok(None),
		}
	}

	async fn save_content_block(
		&self,
		content_block: ContentBlock,
	) -> Result<ContentBlock, ApiError> {
		// Save the content block to memory.
		let mut state = self.state.write().await;
		state.blocks.insert(content_block.id, content_block.clone());

		// Sync content block to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.save_content_block(content_block).await,
			None => Ok(content_block),
		}
	}

	async fn delete_content_block(&self, id: Uuid) -> Result<(), ApiError> {
		// Delete the content block from memory.
		let mut state = self.state.write().await;
		state.blocks.remove(&id);

		// Delete the content block from the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.delete_content_block(id).await,
			None => Ok(()),
		}
	}

	async fn get_content_link(&self, id: Uuid) -> Result<Option<ContentLink>, ApiError> {
		let state = self.state.read().await;

		// Try to find the content link in memory.
		if let Some(link) = state.links.get(&id) {
			return Ok(Some(*link));
		}

		// Try to find the content link in the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.get_content_link(id).await,
			None => Ok(None),
		}
	}

	async fn get_content_links_from(&self, id: Uuid) -> Result<Vec<ContentLink>, ApiError> {
		let state = self.state.read().await;

		// Get all link IDs for this source block.
		let link_ids = state
			.source_block_links
			.get(&id)
			.map(|targets| targets.iter().copied().collect::<Vec<_>>())
			.unwrap_or_default();

		// Get all links from memory.
		let mut links = std::collections::HashMap::new();

		for link_id in link_ids.iter() {
			if let Some(link) = state.links.get(link_id) {
				links.insert(link.id, *link);
			}
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
		let state = self.state.read().await;

		// Get all link IDs for this target block.
		let link_ids = state
			.target_block_links
			.get(&id)
			.map(|sources| sources.iter().copied().collect::<Vec<_>>())
			.unwrap_or_default();

		// Get all links from memory.
		let mut links = std::collections::HashMap::new();

		for link_id in link_ids.iter() {
			if let Some(link) = state.links.get(link_id) {
				links.insert(link.id, *link);
			}
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
		let mut state = self.state.write().await;

		// Save the content link to memory.
		state.links.insert(link.id, link);

		// Update the block links.
		state
			.source_block_links
			.entry(link.source_id)
			.or_insert_with(HashSet::new)
			.insert(link.id);

		state
			.target_block_links
			.entry(link.target_id)
			.or_insert_with(HashSet::new)
			.insert(link.id);

		// Sync content link to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.save_content_link(link).await,
			None => Ok(()),
		}
	}

	async fn delete_content_link(&self, link: ContentLink) -> Result<(), ApiError> {
		let mut state = self.state.write().await;

		// Delete the content link from memory.
		state.links.remove(&link.id);

		// Update the block links.
		if let Some(links) = state.source_block_links.get_mut(&link.source_id) {
			links.remove(&link.id);
			if links.is_empty() {
				state.source_block_links.remove(&link.source_id);
			}
		}

		if let Some(links) = state.target_block_links.get_mut(&link.target_id) {
			links.remove(&link.id);
			if links.is_empty() {
				state.target_block_links.remove(&link.target_id);
			}
		}

		// Sync content link to the linked repository.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => linked_repository.delete_content_link(link).await,
			None => Ok(()),
		}
	}

	async fn are_blocks_linked(&self, source_id: Uuid, target_id: Uuid) -> Result<bool, ApiError> {
		let state = self.state.read().await;

		// Check if there's a link from source to target in memory.
		let has_link = state
			.source_block_links
			.get(&source_id)
			.is_some_and(|link_ids| {
				link_ids.iter().any(|link_id| {
					state
						.links
						.get(link_id)
						.is_some_and(|link| link.target_id == target_id)
				})
			});

		if has_link {
			return Ok(true);
		}

		// Check the linked repository if no link was found.
		match &*self.linked_repository.read().await {
			Some(linked_repository) => {
				linked_repository
					.are_blocks_linked(source_id, target_id)
					.await
			}
			None => Ok(false),
		}
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

	struct MemoryRepositoryFactory;

	impl TestRepositoryFactory for MemoryRepositoryFactory {
		type Repository = MemoryContentRepository;

		fn create_repository(&self) -> Self::Repository {
			MemoryContentRepository::new()
		}
	}

	#[tokio::test]
	async fn test_memory_repository() {
		test_content_repository(MemoryRepositoryFactory).await;
	}
}
