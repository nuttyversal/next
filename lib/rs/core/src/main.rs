use serde::{Deserialize, Serialize};
use sqlx::{
	postgres::PgPoolOptions,
	types::{Json, Uuid},
};

/// A block of content in the Nuttyverse.
#[derive(Debug)]
struct ContentBlock {
	id: Uuid,
	parent_id: Option<Uuid>,
	content: Json<BlockContent>,
}

/// A link between two blocks of content.
struct ContentLink {
	id: Uuid,
	source_id: Uuid,
	target_id: Uuid,
}

/// Not to be confused with [ContentBlock].
/// `ContentBlockContent` it might have been named,
/// but `BlockContent` is shorter, less blamed.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
enum BlockContent {
	Page { title: String },
	Heading { markdown: String },
	Paragraph { markdown: String },
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect("postgres://nutty@localhost:5432/nuttyverse")
		.await?;

	let content_blocks = sqlx::query_as!(
		ContentBlock,
		r#"
			SELECT
				id,
				parent_id,
				-- This type annotation is required to tell SQLx how
				-- to deserialize the JSON column into the enum.
				content AS "content: Json<BlockContent>"
			FROM blocks;
		"#
	)
	.fetch_all(&pool)
	.await?;

	for block in content_blocks {
		println!("{:?}", block);
	}

	Ok(())
}
