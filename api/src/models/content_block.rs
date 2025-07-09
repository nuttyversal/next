use chrono::Local;
use chrono::TimeZone;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use thiserror::Error;

use crate::models::BlockContent;
use crate::models::FractionalIndex;
use crate::models::NuttyId;
use crate::models::date_time_rfc_3339::DateTimeRfc3339;

/// A block of content in the Nuttyverse.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentBlock {
	#[sqlx(rename = "id")]
	nutty_id: NuttyId,
	pub owner_id: Option<NuttyId>,
	pub parent_id: Option<NuttyId>,
	pub f_index: FractionalIndex,
	#[sqlx(json)]
	pub content: BlockContent,
	created_at: DateTimeRfc3339,
	updated_at: DateTimeRfc3339,
}

impl ContentBlock {
	/// Create a new content block.
	fn new(
		nutty_id: NuttyId,
		owner_id: Option<NuttyId>,
		parent_id: Option<NuttyId>,
		f_index: FractionalIndex,
		content: BlockContent,
		created_at: DateTimeRfc3339,
		updated_at: DateTimeRfc3339,
	) -> Self {
		Self {
			nutty_id,
			owner_id,
			parent_id,
			f_index,
			content,
			created_at,
			updated_at,
		}
	}

	/// Create a new content block with a generated identifier (UUIDv7) — right now!
	pub fn now(parent_id: Option<NuttyId>, f_index: FractionalIndex, content: BlockContent) -> Self {
		let nutty_id = NuttyId::now();
		let timestamp = nutty_id.timestamp() as i64;

		let now = Local
			.timestamp_millis_opt(timestamp)
			.single()
			.unwrap()
			.fixed_offset()
			.into();

		Self::new(NuttyId::now(), None, parent_id, f_index, content, now, now)
	}

	/// Create a new content block with an owner.
	pub fn now_with_owner(
		parent_id: Option<NuttyId>,
		owner_id: NuttyId,
		f_index: FractionalIndex,
		content: BlockContent,
	) -> Self {
		let nutty_id = NuttyId::now();
		let timestamp = nutty_id.timestamp() as i64;

		let now = Local
			.timestamp_millis_opt(timestamp)
			.single()
			.unwrap()
			.fixed_offset()
			.into();

		Self::new(
			NuttyId::now(),
			Some(owner_id),
			parent_id,
			f_index,
			content,
			now,
			now,
		)
	}

	/// Get the Nutty ID.
	pub fn nutty_id(&self) -> &NuttyId {
		&self.nutty_id
	}

	/// Get the owner ID.
	pub fn owner_id(&self) -> Option<&NuttyId> {
		self.owner_id.as_ref()
	}

	/// Check if the content block is owned by the given navigator.
	pub fn is_owned_by(&self, navigator_id: &NuttyId) -> bool {
		self
			.owner_id
			.as_ref()
			.map_or(false, |owner| owner == navigator_id)
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
	owner_id: Option<NuttyId>,
	parent_id: Option<NuttyId>,
	f_index: Option<FractionalIndex>,
	content: Option<BlockContent>,
	created_at: Option<DateTimeRfc3339>,
	updated_at: Option<DateTimeRfc3339>,
}

impl ContentBlockBuilder {
	/// Set the Nutty ID.
	pub fn nutty_id(mut self, nutty_id: NuttyId) -> Self {
		self.nutty_id = Some(nutty_id);
		self
	}

	/// Set the owner ID.
	pub fn owner_id(mut self, owner_id: Option<NuttyId>) -> Self {
		self.owner_id = owner_id;
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
	pub fn created_at(mut self, created_at: DateTimeRfc3339) -> Self {
		self.created_at = Some(created_at);
		self
	}

	/// Set the "updated at" time.
	pub fn updated_at(mut self, updated_at: DateTimeRfc3339) -> Self {
		self.updated_at = Some(updated_at);
		self
	}

	/// Build the content block, returning an error if required fields are not set.
	pub fn try_build(self) -> Result<ContentBlock, ContentBlockBuilderError> {
		let parent_id = self.parent_id;
		let owner_id = self.owner_id;
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
					nutty_id, owner_id, parent_id, f_index, content, created_at, updated_at,
				))
			}

			// … or with no timestamps at all. Generate them on the fly.
			(None, None, None) => {
				if let Some(owner_id) = owner_id {
					Ok(ContentBlock::now_with_owner(
						parent_id, owner_id, f_index, content,
					))
				} else {
					Ok(ContentBlock::now(parent_id, f_index, content))
				}
			}

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_content_block_with_owner() {
		let owner_id = NuttyId::now();
		let content = BlockContent::Page {
			title: "Test Page".to_string(),
		};

		let block = ContentBlock::now_with_owner(None, owner_id, FractionalIndex::start(), content);

		assert_eq!(block.owner_id(), Some(&owner_id));
		assert!(block.is_owned_by(&owner_id));

		let different_owner = NuttyId::now();
		assert!(!block.is_owned_by(&different_owner));
	}

	#[test]
	fn test_content_block_without_owner() {
		let content = BlockContent::Page {
			title: "Test Page".to_string(),
		};

		let block = ContentBlock::now(None, FractionalIndex::start(), content);

		assert_eq!(block.owner_id(), None);
		assert!(!block.is_owned_by(&NuttyId::now()));
	}

	#[test]
	fn test_content_block_builder_with_owner() {
		let owner_id = NuttyId::now();
		let content = BlockContent::Page {
			title: "Test Page".to_string(),
		};

		let block = ContentBlock::builder()
			.f_index(FractionalIndex::start())
			.content(content)
			.owner_id(Some(owner_id))
			.try_build()
			.expect("Failed to build content block");

		assert_eq!(block.owner_id(), Some(&owner_id));
		assert!(block.is_owned_by(&owner_id));
	}
}
