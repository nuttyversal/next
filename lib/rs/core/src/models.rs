use crate::index::FractionalIndex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Not to be confused with [ContentBlock].
/// `ContentBlockContent` it might have been named,
/// but `BlockContent` is shorter and unclaimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BlockContent {
	Page { title: String },
	Heading { markdown: String },
	Paragraph { markdown: String },
}

/// A block of content in the Nuttyverse.
#[derive(Debug, Clone)]
pub struct ContentBlock {
	pub id: Uuid,
	pub parent_id: Option<Uuid>,
	pub content: BlockContent,
	pub index: FractionalIndex,
}

impl ContentBlock {
	/// Create a new block of content.
	pub fn new(id: Uuid, parent_id: Option<Uuid>, content: BlockContent, index: FractionalIndex) -> Self {
		Self {
			id,
			parent_id,
			content,
			index,
		}
	}

	/// Create a new block of content with a generated identifier (UUIDv7).
	pub fn now(parent_id: Option<Uuid>, content: BlockContent, index: FractionalIndex) -> Self {
		Self::new(Uuid::now_v7(), parent_id, content, index)
	}

	/// Serialize content to a JSON value.
	pub fn serialize_content(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(&self.content)
	}

	/// Deserialize content from a JSON value.
	pub fn deserialize_content(
		content: serde_json::Value,
	) -> Result<BlockContent, serde_json::Error> {
		serde_json::from_value(content)
	}

	/// Create a builder for a new content block.
	pub fn builder() -> ContentBlockBuilder {
		ContentBlockBuilder::default()
	}
}

/// Builder for creating new content blocks.
#[derive(Default)]
pub struct ContentBlockBuilder {
	id: Option<Uuid>,
	parent_id: Option<Uuid>,
	content: Option<BlockContent>,
	index: Option<FractionalIndex>,
}

impl ContentBlockBuilder {
	/// Set the block's ID.
	pub fn id(mut self, id: Uuid) -> Self {
		self.id = Some(id);
		self
	}

	/// Set the block's parent ID.
	pub fn parent_id(mut self, parent_id: Option<Uuid>) -> Self {
		self.parent_id = parent_id;
		self
	}

	/// Set the block's content.
	pub fn content(mut self, content: BlockContent) -> Self {
		self.content = Some(content);
		self
	}

	/// Set the block's index.
	pub fn index(mut self, index: FractionalIndex) -> Self {
		self.index = Some(index);
		self
	}

	/// Build the content block, returning an error if required fields are not set.
	pub fn try_build(self) -> Result<ContentBlock, ContentBlockError> {
		Ok(ContentBlock {
			id: self.id.unwrap_or_else(Uuid::now_v7),
			parent_id: self.parent_id,
			content: self.content.ok_or(ContentBlockError::MissingContent)?,
			index: self.index.ok_or(ContentBlockError::MissingIndex)?,
		})
	}
}

/// Errors that can occur when building a content block.
#[derive(Debug, Error)]
pub enum ContentBlockError {
	#[error("Content is required")]
	MissingContent,

	#[error("Index is required")]
	MissingIndex,
}

/// A link between two blocks of content.
#[derive(Debug, Copy, Clone)]
pub struct ContentLink {
	pub id: Uuid,
	pub source_id: Uuid,
	pub target_id: Uuid,
}

impl ContentLink {
	pub fn new(id: Uuid, source_id: Uuid, target_id: Uuid) -> Self {
		Self { id, source_id, target_id }
	}
}

impl ContentLink {
	pub fn now(source_id: Uuid, target_id: Uuid) -> Self {
		Self::new(Uuid::now_v7(), source_id, target_id)
	}
}
