import { ParseResult, Schema } from "effect";
import { Temporal } from "temporal-polyfill";

/**
 * Schema transformation between `Temporal.Instant` and `string`.
 */
const InstantFromString = Schema.transformOrFail(
	Schema.String,
	Schema.instanceOf(Temporal.Instant),
	{
		strict: true,
		decode: (input, _options, ast) => {
			try {
				return ParseResult.succeed(Temporal.Instant.from(input));
			} catch (error) {
				return ParseResult.fail(
					new ParseResult.Type(ast, input, `${error}`),
				);
			}
		},
		encode: (input, _options, _ast) => {
			return ParseResult.succeed(input.toString());
		},
	},
);

export { InstantFromString };
