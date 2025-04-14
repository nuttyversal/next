use crate::models::NuttyTag;
use serde::{Deserialize, Serialize};

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

impl BlockContent {
	/// Parse the target [NuttyTag] list from the content block.
	pub fn parse_target_tags(&self) -> Vec<NuttyTag> {
		match self {
			BlockContent::Page { .. } => vec![],
			BlockContent::Heading { markdown } => NuttyTag::parse_all(markdown),
			BlockContent::Paragraph { markdown } => NuttyTag::parse_all(markdown),
		}
	}
}
