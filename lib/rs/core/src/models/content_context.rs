use crate::models::{ContentBlock, NuttyId};
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

/// Represents the immediate context of a content block.
///
/// This structure should contain the necessary information for the initial
/// rendering of a content block. When constructing the context, take care
/// to ensure that the following content blocks are included in the cache:
///
/// ```text
///             • … and so on, and so forth — all ancestor content blocks, if any.
///          • The parent of the parent of the parent content block, if any.
///       • The parent of the parent content block, if any.
///    • The parent content block, if any.
/// • The content block, itself.
///    • The children content blocks, if any.
///       • The children of the children content blocks, if any.
///          • The children of the children of the children content blocks, if any.
///             • … and so on, and so forth — all descendent content blocks, if any.
///
/// • The reference (outbound links) content blocks, if any.
/// • The backlinked (inbound links) content blocks, if any.
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ContentContext {
	/// The Nutty ID of the content block.
	block_id: NuttyId,

	/// The Nutty ID of the parent content block, if any.
	parent_id: Option<NuttyId>,

	/// A list of Nutty IDs of child content blocks.
	children_ids: Vec<NuttyId>,

	/// A list of Nutty IDs of content blocks that this block references.
	reference_ids: Vec<NuttyId>,

	/// A list of Nutty IDs of content blocks that reference this block.
	backlink_ids: Vec<NuttyId>,

	/// A cache of content blocks for quick access.
	block_cache: HashMap<NuttyId, ContentBlock>,
}

impl ContentContext {
	/// Create a new content context.
	fn new(
		block_id: NuttyId,
		parent_id: Option<NuttyId>,
		children_ids: Vec<NuttyId>,
		reference_ids: Vec<NuttyId>,
		backlink_ids: Vec<NuttyId>,
		block_cache: HashMap<NuttyId, ContentBlock>,
	) -> Self {
		Self {
			block_id,
			parent_id,
			children_ids,
			reference_ids,
			backlink_ids,
			block_cache,
		}
	}

	/// Get the block ID.
	pub fn block_id(&self) -> &NuttyId {
		&self.block_id
	}

	/// Get the parent ID.
	pub fn parent_id(&self) -> Option<&NuttyId> {
		self.parent_id.as_ref()
	}

	/// Get the children IDs.
	pub fn children_ids(&self) -> &[NuttyId] {
		&self.children_ids
	}

	/// Get the reference IDs.
	pub fn reference_ids(&self) -> &[NuttyId] {
		&self.reference_ids
	}

	/// Get the backlink IDs.
	pub fn backlink_ids(&self) -> &[NuttyId] {
		&self.backlink_ids
	}

	/// Get the block cache.
	pub fn block_cache(&self) -> &HashMap<NuttyId, ContentBlock> {
		&self.block_cache
	}

	/// Create a builder for a new content context.
	pub fn builder() -> ContentContextBuilder {
		ContentContextBuilder::default()
	}
}

/// A builder for creating new content contexts.
#[derive(Default)]
pub struct ContentContextBuilder {
	block_id: Option<NuttyId>,
	parent_id: Option<NuttyId>,
	children_ids: Vec<NuttyId>,
	reference_ids: Vec<NuttyId>,
	backlink_ids: Vec<NuttyId>,
	block_cache: HashMap<NuttyId, ContentBlock>,
}

impl ContentContextBuilder {
	/// Set the block ID.
	pub fn block_id(mut self, block_id: NuttyId) -> Self {
		self.block_id = Some(block_id);
		self
	}

	/// Set the parent ID.
	pub fn parent_id(mut self, parent_id: Option<NuttyId>) -> Self {
		self.parent_id = parent_id;
		self
	}

	/// Set the children IDs.
	pub fn children_ids(mut self, children_ids: Vec<NuttyId>) -> Self {
		self.children_ids = children_ids;
		self
	}

	/// Add a child ID.
	pub fn add_child_id(mut self, child_id: NuttyId) -> Self {
		self.children_ids.push(child_id);
		self
	}

	/// Set the reference IDs.
	pub fn reference_ids(mut self, reference_ids: Vec<NuttyId>) -> Self {
		self.reference_ids = reference_ids;
		self
	}

	/// Add a reference ID.
	pub fn add_reference_id(mut self, reference_id: NuttyId) -> Self {
		self.reference_ids.push(reference_id);
		self
	}

	/// Set the backlink IDs.
	pub fn backlink_ids(mut self, backlink_ids: Vec<NuttyId>) -> Self {
		self.backlink_ids = backlink_ids;
		self
	}

	/// Add a backlink ID.
	pub fn add_backlink_id(mut self, backlink_id: NuttyId) -> Self {
		self.backlink_ids.push(backlink_id);
		self
	}

	/// Set the block cache.
	pub fn block_cache(mut self, block_cache: HashMap<NuttyId, ContentBlock>) -> Self {
		self.block_cache = block_cache;
		self
	}

	/// Add a block to the cache.
	pub fn add_block_to_cache(mut self, block: ContentBlock) -> Self {
		self.block_cache.insert(*block.nutty_id(), block);
		self
	}

	/// Build the content context, returning an error if required fields are not set.
	pub fn try_build(self) -> Result<ContentContext, ContentContextBuilderError> {
		let block_id = self
			.block_id
			.ok_or(ContentContextBuilderError::MissingBlockId)?;
		let parent_id = self.parent_id;
		let children_ids = self.children_ids;
		let reference_ids = self.reference_ids;
		let backlink_ids = self.backlink_ids;
		let block_cache = self.block_cache;

		Ok(ContentContext::new(
			block_id,
			parent_id,
			children_ids,
			reference_ids,
			backlink_ids,
			block_cache,
		))
	}
}

#[derive(Debug, Error)]
pub enum ContentContextBuilderError {
	#[error("Block ID is required")]
	MissingBlockId,
}
