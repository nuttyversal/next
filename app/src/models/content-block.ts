import { Option, Schema } from "effect";
import { Temporal } from "temporal-polyfill";

import { BlockContent } from "./block-content.ts";
import {
	FractionalIndex,
	FractionalIndexFromString,
} from "./fractional-index.ts";
import { InstantFromString } from "./instant.ts";
import { NuttyId, NuttyIdFromString } from "./nutty-id.ts";

/**
 * A block of content in the Nuttyverse.
 */
class ContentBlock extends Schema.Class<ContentBlock>("ContentBlock")({
	nuttyId: NuttyId,

	// `Schema.Option` is a schema transformation.
	// Wrapping it with typeSchema prevents double decoding.
	parentId: Schema.typeSchema(Schema.Option(NuttyId)),

	// Class schemas are also schema transformations.
	// Wrapping it with typeSchema prevents double decoding.
	fIndex: Schema.typeSchema(FractionalIndex),

	content: BlockContent,
	createdAt: Schema.instanceOf(Temporal.Instant),
	updatedAt: Schema.instanceOf(Temporal.Instant),
}) {
	/**
	 * Create a new content block with a generated identifier (UUIDv7) — right now!
	 */
	static now(
		parentId: Option.Option<NuttyId>,
		fIndex: FractionalIndex,
		content: BlockContent,
	): ContentBlock {
		const now = Temporal.Now.instant();

		return new ContentBlock({
			nuttyId: NuttyId.now(),
			parentId,
			fIndex,
			content,
			createdAt: now,
			updatedAt: now,
		});
	}
}

/**
 * Schema transformation between `ContentBlock` and its API model.
 */
const ContentBlockFromApi = Schema.transform(
	Schema.Struct({
		nutty_id: NuttyIdFromString,
		parent_id: Schema.OptionFromNullOr(NuttyIdFromString),
		f_index: FractionalIndexFromString,
		content: BlockContent,
		created_at: InstantFromString,
		updated_at: InstantFromString,
	}),
	ContentBlock,
	{
		strict: true,
		decode: (api) => {
			return new ContentBlock({
				nuttyId: api.nutty_id,
				parentId: api.parent_id,
				fIndex: api.f_index,
				content: api.content,
				createdAt: api.created_at,
				updatedAt: api.updated_at,
			});
		},
		encode: (domain) => {
			return {
				nutty_id: domain.nuttyId,
				parent_id: domain.parentId,
				f_index: domain.fIndex,
				content: domain.content,
				created_at: domain.createdAt,
				updated_at: domain.updatedAt,
			};
		},
	},
);

export { ContentBlock, ContentBlockFromApi };
