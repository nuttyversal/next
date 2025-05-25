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
 * An API response containing errors.
 */
const ErrorResponse = Schema.Struct({
	errors: Schema.Array(ApiError),
}).annotations({ identifier: "ErrorResponse" });

type ErrorResponse = typeof ErrorResponse.Type;

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
 * Either a `SingleResponse` or an `ErrorResponse`.
 */
const SingleOrErrorResponse = <A, I>(schema: Schema.Schema<A, I>) => {
	return Schema.Union(SingleResponse(schema), ErrorResponse).pipe(
		Schema.annotations({ identifier: "SingleResponseOrError" }),
	);
};

type SingleOrErrorResponse<A, I> = Schema.Schema.Type<
	ReturnType<typeof SingleOrErrorResponse<A, I>>
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
 * Either a `MultipleResponse` or an `ErrorResponse`.
 */
const MultipleOrErrorResponse = <A, I>(schema: Schema.Schema<A, I>) => {
	return Schema.Union(MultipleResponse(schema), ErrorResponse).pipe(
		Schema.annotations({ identifier: "MultipleResponseOrError" }),
	);
};

type MultipleOrErrorResponse<A, I> = Schema.Schema.Type<
	ReturnType<typeof MultipleOrErrorResponse<A, I>>
>;

/**
 * Pattern match against an API response.
 */
function match<A, I, D, E = D>(
	self: SingleOrErrorResponse<A, I>,
	options: {
		readonly onData: (data: A | null) => D;
		readonly onError: (errors: readonly ApiError[]) => E;
	},
): D | E;
function match<A, I, D, E = D>(
	self: MultipleOrErrorResponse<A, I>,
	options: {
		readonly onData: (data: A[] | null) => D;
		readonly onError: (errors: readonly ApiError[]) => E;
	},
): D | E;
function match<A, I, D, E = D>(
	self: SingleOrErrorResponse<A, I> | MultipleOrErrorResponse<A, I>,
	options: {
		readonly onData: (data: A | readonly A[] | null) => D;
		readonly onError: (errors: readonly ApiError[]) => E;
	},
): D | E {
	if ("data" in self) {
		return options.onData(self.data);
	} else {
		return options.onError(self.errors);
	}
}

/**
 * A namespace for API response utility functions.
 */
const Response = {
	match,
};

export {
	ApiError,
	ErrorResponse,
	MultipleOrErrorResponse,
	MultipleResponse,
	Response,
	SingleOrErrorResponse,
	SingleResponse,
};
