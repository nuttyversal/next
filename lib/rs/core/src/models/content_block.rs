use crate::models::{BlockContent, FractionalIndex, NuttyId};
use thiserror::Error;

/// A block of content in the Nuttyverse.
#[derive(Debug, Clone)]
pub struct ContentBlock {
	nutty_id: NuttyId,
	pub parent_id: Option<NuttyId>,
	pub content: BlockContent,
	pub index: FractionalIndex,
}

impl ContentBlock {
	/// Create a new content block.
	fn new(
		nutty_id: NuttyId,
		parent_id: Option<NuttyId>,
		content: BlockContent,
		index: FractionalIndex,
	) -> Self {
		Self {
			nutty_id,
			parent_id,
			content,
			index,
		}
	}

	/// Create a new content block with a generated identifier (UUIDv7).
	pub fn now(parent_id: Option<NuttyId>, content: BlockContent, index: FractionalIndex) -> Self {
		Self::new(NuttyId::now(), parent_id, content, index)
	}

	/// Get the Nutty ID.
	pub fn nutty_id(&self) -> &NuttyId {
		&self.nutty_id
	}

	/// Serialize content to a JSON value.
	pub fn serialize_content(&self) -> Result<serde_json::Value, ContentBlockError> {
		serde_json::to_value(self.content.clone()).map_err(ContentBlockError::SerializationError)
	}

	/// Deserialize content from a JSON value.
	pub fn deserialize_content(
		content: serde_json::Value,
	) -> Result<BlockContent, ContentBlockError> {
		serde_json::from_value(content).map_err(ContentBlockError::DeserializationError)
	}

	/// Create a builder for a new content block.
	pub fn builder() -> ContentBlockBuilder {
		ContentBlockBuilder::default()
	}
}

#[derive(Debug, Error)]
pub enum ContentBlockError {
	#[error("SerializationError: {0}")]
	SerializationError(serde_json::Error),

	#[error("DeserializationError: {0}")]
	DeserializationError(serde_json::Error),
}

/// A builder for creating new content blocks.
#[derive(Default)]
pub struct ContentBlockBuilder {
	nutty_id: Option<NuttyId>,
	parent_id: Option<NuttyId>,
	content: Option<BlockContent>,
	index: Option<FractionalIndex>,
}

impl ContentBlockBuilder {
	/// Set the Nutty ID.
	pub fn nutty_id(mut self, nutty_id: NuttyId) -> Self {
		self.nutty_id = Some(nutty_id);
		self
	}

	/// Set the parent Nutty ID.
	pub fn parent_id(mut self, parent_id: Option<NuttyId>) -> Self {
		self.parent_id = parent_id;
		self
	}

	/// Set the block content.
	pub fn content(mut self, content: BlockContent) -> Self {
		self.content = Some(content);
		self
	}

	/// Set the positional index.
	pub fn index(mut self, index: FractionalIndex) -> Self {
		self.index = Some(index);
		self
	}

	/// Build the content block, returning an error if required fields are not set.
	pub fn try_build(self) -> Result<ContentBlock, ContentBlockBuilderError> {
		let nutty_id = self.nutty_id.unwrap_or_else(NuttyId::now);
		let parent_id = self.parent_id;
		let content = self
			.content
			.ok_or(ContentBlockBuilderError::MissingContent)?;
		let index = self.index.ok_or(ContentBlockBuilderError::MissingIndex)?;

		Ok(ContentBlock::new(nutty_id, parent_id, content, index))
	}
}

#[derive(Debug, Error)]
pub enum ContentBlockBuilderError {
	#[error("Content is required")]
	MissingContent,

	#[error("Index is required")]
	MissingIndex,
}
