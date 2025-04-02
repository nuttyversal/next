use serde::{Deserialize, Serialize};
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
}

impl ContentBlock {
	/// Create a new block of content.
	pub fn new(id: Uuid, parent_id: Option<Uuid>, content: BlockContent) -> Self {
		Self {
			id,
			parent_id,
			content,
		}
	}

	/// Create a new block of content with a generated identifier (UUIDv7).
	pub fn now(parent_id: Option<Uuid>, content: BlockContent) -> Self {
		Self::new(Uuid::now_v7(), parent_id, content)
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
}

/// A link between two blocks of content.
pub struct ContentLink {
	pub id: Uuid,
	pub source_id: Uuid,
	pub target_id: Uuid,
}
