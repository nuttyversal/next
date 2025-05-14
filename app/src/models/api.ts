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
const SingleResponse = <T>(schema: Schema.Schema<T>) => {
	return Schema.Struct({
		data: Schema.NullOr(schema),
	}).pipe(Schema.annotations({ identifier: "SingleResponse" }));
};

type SingleResponse<T> = Schema.Schema.Type<
	ReturnType<typeof SingleResponse<T>>
>;

/**
 * An API response containing multiple resource objects.
 */
const MultipleResponse = <T>(schema: Schema.Schema<T>) => {
	return Schema.Struct({
		data: Schema.Array(schema),
	}).pipe(Schema.annotations({ identifier: "MultipleResponse" }));
};

type MultipleResponse<T> = Schema.Schema.Type<
	ReturnType<typeof MultipleResponse<T>>
>;

/**
 * An API response containing errors.
 */
const ErrorResponse = Schema.Struct({
	errors: Schema.Array(ApiError),
}).annotations({ identifier: "ErrorResponse" });

type ErrorResponse = typeof ErrorResponse.Type;

export { ApiError, ErrorResponse, MultipleResponse, SingleResponse };
