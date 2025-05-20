import { Schema } from "effect";
import { Temporal } from "temporal-polyfill";

import { InstantFromString } from "./instant.ts";
import { NuttyId, NuttyIdFromString } from "./nutty-id.ts";

/**
 * A user session in the Nuttyverse.
 */
const Session = Schema.Struct({
	navigatorId: NuttyId,
	expiresAt: Schema.instanceOf(Temporal.Instant),
	createdAt: Schema.instanceOf(Temporal.Instant),
	updatedAt: Schema.instanceOf(Temporal.Instant),
}).pipe(
	Schema.annotations({
		identifier: "Session",
		title: "Session",
		description: "A user session in the Nuttyverse",
	}),
);

type Session = typeof Session.Type;

/**
 * Schema transformation between `Session` and its API model.
 */
const SessionFromApi = Schema.transform(
	Schema.Struct({
		navigator_id: NuttyIdFromString,
		expires_at: InstantFromString,
		created_at: InstantFromString,
		updated_at: InstantFromString,
	}),
	Session,
	{
		strict: true,
		decode: (api) => ({
			navigatorId: api.navigator_id,
			expiresAt: api.expires_at,
			createdAt: api.created_at,
			updatedAt: api.updated_at,
		}),
		encode: (domain) => ({
			navigator_id: domain.navigatorId,
			expires_at: domain.expiresAt,
			created_at: domain.createdAt,
			updated_at: domain.updatedAt,
		}),
	},
);

export { Session, SessionFromApi };
