import { Schema } from "effect";

const PrettyError = Schema.Struct({
	what: Schema.String,
	why: Schema.String,
	trace: Schema.Array(Schema.String),
});

type PrettyError = typeof PrettyError.Type;

export { PrettyError };
