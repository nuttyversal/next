use serde::Deserialize;
use serde::Serialize;
use sqlx::Decode;
use sqlx::Encode;
use sqlx::FromRow;
use sqlx::Postgres;
use sqlx::Row;
use sqlx::Type;
use sqlx::postgres::PgRow;
use sqlx::postgres::PgTypeInfo;

use crate::models::NuttyTag;

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

impl FromRow<'_, PgRow> for BlockContent {
	fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
		let content = row.try_get("content")?;

		serde_json::from_value(content).map_err(|e| sqlx::Error::ColumnDecode {
			index: "content".to_string(),
			source: Box::new(e),
		})
	}
}

impl Type<Postgres> for BlockContent {
	fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
		PgTypeInfo::with_name("JSONB")
	}
}

impl Encode<'_, sqlx::Postgres> for BlockContent {
	fn encode_by_ref(
		&self,
		buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'_>,
	) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
		let json = serde_json::to_string(self).unwrap();
		<String as sqlx::encode::Encode<sqlx::Postgres>>::encode(json, buf)
	}
}

impl<'r> Decode<'r, sqlx::Postgres> for BlockContent {
	fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
		let value = <&str as sqlx::decode::Decode<sqlx::Postgres>>::decode(value)?;
		let block_content = serde_json::from_str(value)?;
		Ok(block_content)
	}
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
