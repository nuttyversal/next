import { Schema } from "effect";

/**
 * Schema for content block content.
 *
 * Not to be confused with ContentBlock.
 * `ContentBlockContent` it might have been named,
 * but `BlockContent` is shorter and unclaimed.
 */
const BlockContent = Schema.Union(
	Schema.Struct({
		kind: Schema.Literal("Page"),
		title: Schema.String,
	}),
	Schema.Struct({
		kind: Schema.Literal("Heading"),
		markdown: Schema.String,
	}),
	Schema.Struct({
		kind: Schema.Literal("Paragraph"),
		markdown: Schema.String,
	}),
).pipe(
	Schema.annotations({
		identifier: "BlockContent",
		title: "Block Content",
		description: "Content for a block in the Nuttyverse",
	}),
);

type BlockContent = typeof BlockContent.Type;

export { BlockContent };
