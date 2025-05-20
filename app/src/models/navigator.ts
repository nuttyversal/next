import { Schema } from "effect";
import { Temporal } from "temporal-polyfill";

import { InstantFromString } from "./instant.ts";
import { NuttyId, NuttyIdFromString } from "./nutty-id.ts";

/**
 * A schema that encodes the validation rules for a `Navigator` name.
 */
const NavigatorName = Schema.String.pipe(
	Schema.minLength(4),
	Schema.maxLength(16),
	Schema.pattern(/^[a-z0-9_]+$/),
);

type NavigatorName = typeof NavigatorName.Type;

/**
 * A navigator in the Nuttyverse.
 */
const Navigator = Schema.Struct({
	nuttyId: NuttyId,
	name: NavigatorName,
	createdAt: Schema.instanceOf(Temporal.Instant),
	updatedAt: Schema.instanceOf(Temporal.Instant),
}).pipe(
	Schema.annotations({
		identifier: "Navigator",
		title: "Navigator",
		description: "A navigator in the Nuttyverse",
	}),
);

type Navigator = typeof Navigator.Type;

/**
 * Schema transformation between `Navigator` and its API model.
 */
const NavigatorFromApi = Schema.transform(
	Schema.Struct({
		nutty_id: NuttyIdFromString,
		name: Schema.String,
		created_at: InstantFromString,
		updated_at: InstantFromString,
	}),
	Navigator,
	{
		strict: true,
		decode: (api) => ({
			nuttyId: api.nutty_id,
			name: api.name,
			createdAt: api.created_at,
			updatedAt: api.updated_at,
		}),
		encode: (domain) => ({
			nutty_id: domain.nuttyId,
			name: domain.name,
			created_at: domain.createdAt,
			updated_at: domain.updatedAt,
		}),
	},
);

export { Navigator, NavigatorFromApi, NavigatorName };
