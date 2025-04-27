use crate::models::{BlockContent, FractionalIndex, NuttyId};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// A block of content in the Nuttyverse.
#[derive(Debug, Clone)]
pub struct ContentBlock {
	nutty_id: NuttyId,
	pub parent_id: Option<NuttyId>,
	pub f_index: FractionalIndex,
	pub content: BlockContent,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl ContentBlock {
	/// Create a new content block.
	fn new(
		nutty_id: NuttyId,
		parent_id: Option<NuttyId>,
		f_index: FractionalIndex,
		content: BlockContent,
		created_at: DateTime<Utc>,
		updated_at: DateTime<Utc>,
	) -> Self {
		Self {
			nutty_id,
			parent_id,
			f_index,
			content,
			created_at,
			updated_at,
		}
	}

	/// Create a new content block with a generated identifier (UUIDv7) — right now!
	pub fn now(parent_id: Option<NuttyId>, f_index: FractionalIndex, content: BlockContent) -> Self {
		let now = Utc::now();
		Self::new(NuttyId::now(), parent_id, f_index, content, now, now)
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
	f_index: Option<FractionalIndex>,
	content: Option<BlockContent>,
	created_at: Option<DateTime<Utc>>,
	updated_at: Option<DateTime<Utc>>,
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

	/// Set the fractional index.
	pub fn f_index(mut self, f_index: FractionalIndex) -> Self {
		self.f_index = Some(f_index);
		self
	}

	/// Set the block content.
	pub fn content(mut self, content: BlockContent) -> Self {
		self.content = Some(content);
		self
	}

	/// Set the "created at" time.
	pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
		self.created_at = Some(created_at);
		self
	}

	/// Set the "updated at" time.
	pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
		self.updated_at = Some(updated_at);
		self
	}

	/// Build the content block, returning an error if required fields are not set.
	pub fn try_build(self) -> Result<ContentBlock, ContentBlockBuilderError> {
		let parent_id = self.parent_id;
		let f_index = self.f_index.ok_or(ContentBlockBuilderError::MissingIndex)?;

		let content = self
			.content
			.ok_or(ContentBlockBuilderError::MissingContent)?;

		match (self.nutty_id, self.created_at, self.updated_at) {
			// Either create the content block with all timestamps …
			(Some(nutty_id), Some(created_at), Some(updated_at)) => {
				if updated_at < created_at {
					return Err(ContentBlockBuilderError::InvalidUpdatedAt);
				}

				Ok(ContentBlock::new(
					nutty_id, parent_id, f_index, content, created_at, updated_at,
				))
			}

			// … or with no timestamps at all. Generate them on the fly.
			(None, None, None) => Ok(ContentBlock::now(parent_id, f_index, content)),

			// But, don't create the content block with partial timestamp context.
			(_, _, _) => Err(ContentBlockBuilderError::PartialTimestampContext),
		}
	}
}

#[derive(Debug, Error)]
pub enum ContentBlockBuilderError {
	#[error("Content is required")]
	MissingContent,

	#[error("Index is required")]
	MissingIndex,

	#[error("Invalid 'updated_at' value: Must be >= 'created_at'")]
	InvalidUpdatedAt,

	#[error("Missing partial timestamp context")]
	PartialTimestampContext,
}
