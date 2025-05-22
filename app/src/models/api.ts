import { Schema } from "effect";

/**
 * An API error object.
 */
const ApiError = Schema.Struct({
	code: Schema.optional(Schema.String),
	trace: Schema.Array(Schema.String),
	message: Schema.optional(Schema.String),
	summary: Schema.optional(Schema.String),
});

type ApiError = typeof ApiError.Type;

/**
 * An API response containing a (maybe) single resource object.
 */
const SingleResponse = <A, I>(schema: Schema.Schema<A, I>) => {
	return Schema.Struct({
		data: Schema.NullOr(schema),
	}).pipe(Schema.annotations({ identifier: "SingleResponse" }));
};

type SingleResponse<A, I> = Schema.Schema.Type<
	ReturnType<typeof SingleResponse<A, I>>
>;

/**
 * An API response containing multiple resource objects.
 */
const MultipleResponse = <A, I>(schema: Schema.Schema<A, I>) => {
	return Schema.Struct({
		data: Schema.Array(schema),
	}).pipe(Schema.annotations({ identifier: "MultipleResponse" }));
};

type MultipleResponse<A, I> = Schema.Schema.Type<
	ReturnType<typeof MultipleResponse<A, I>>
>;

/**
 * An API response containing errors.
 */
const ErrorResponse = Schema.Struct({
	errors: Schema.Array(ApiError),
}).annotations({ identifier: "ErrorResponse" });

type ErrorResponse = typeof ErrorResponse.Type;

export { ApiError, ErrorResponse, MultipleResponse, SingleResponse };
