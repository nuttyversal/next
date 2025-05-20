import { Either, Option, Schema } from "effect";
import { Temporal } from "temporal-polyfill";

import { ContentBlock, ContentBlockFromApi } from "./content-block.ts";
import { FractionalIndex } from "./fractional-index.ts";
import { NuttyId } from "./nutty-id.ts";

describe("ContentBlockFromApi", () => {
	test("should decode JSON API model to ContentBlock", () => {
		// Arrange: Example JSON input.
		const jsonInput = {
			nutty_id: "1CPGPsiKYQXGKEigmpvxce:Zb9yCfY",
			parent_id: null,
			f_index: "!",
			content: {
				kind: "Page",
				title: "Parent Page",
			},
			created_at: "2025-05-19T06:12:55.907189+00:00",
			updated_at: "2025-05-19T06:12:55.907189+00:00",
		};

		// Act: Decode the JSON to a ContentBlock.
		const contentBlockApi =
			Schema.decodeUnknownEither(ContentBlockFromApi)(jsonInput);

		// Assert: Decoding was successful.
		expect(Either.isRight(contentBlockApi)).toBe(true);
	});

	test("should encode ContentBlock to JSON API model", () => {
		// Arrange: Create content block.
		const contentBlock = new ContentBlock({
			nuttyId: NuttyId.now(),
			parentId: Option.none(),
			fIndex: FractionalIndex.make({
				index: "!",
			}),
			content: {
				kind: "Page",
				title: "Parent Page",
			},
			createdAt: Temporal.Instant.from("2025-05-19T06:12:55.907189+00:00"),
			updatedAt: Temporal.Instant.from("2025-05-19T06:12:55.907189+00:00"),
		});

		// Act: Encode the ContentBlock to JSON.
		const jsonOutput =
			Schema.encodeUnknownEither(ContentBlockFromApi)(contentBlock);

		// Assert: Encoding was successful.
		expect(Either.isRight(jsonOutput)).toBe(true);
	});
});
