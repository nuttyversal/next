import { Schema } from "effect";
import { describe, expect, it } from "vitest";

import { ErrorResponse, MultipleResponse, SingleResponse } from "./api.ts";

const Message = Schema.Struct({
	id: Schema.String,
	message: Schema.String,
});

type Message = typeof Message.Type;

describe("ApiError Schema", () => {
	it("decodes a single response", () => {
		// Arrange.
		const validResponses = [
			{
				data: null,
			},
			{
				data: {
					id: "1",
					message: "test",
				},
			},
		];

		validResponses.forEach((error) => {
			// Act.
			const result = Schema.decodeUnknownEither(SingleResponse(Message))(
				error,
			);

			// Assert.
			expect(result._tag).toBe("Right");
		});
	});

	it("decodes a multiple response", () => {
		// Arrange.
		const validResponses = [
			{
				data: [],
			},
			{
				data: [
					{
						id: "1",
						message: "hello",
					},
					{
						id: "2",
						message: "world",
					},
				],
			},
		];

		validResponses.forEach((error) => {
			// Act.
			const result = Schema.decodeUnknownEither(MultipleResponse(Message))(
				error,
			);

			// Assert.
			expect(result._tag).toBe("Right");
		});
	});

	it("decodes an error response", () => {
		// Arrange.
		const validResponses = [
			{
				errors: [],
			},
			{
				errors: [
					{
						code: "SessionError",
						trace: ["SessionError"],
						message: "Missing cookie",
						summary: "No session cookie found.",
					},
				],
			},
		];

		validResponses.forEach((error) => {
			// Act.
			const result = Schema.decodeUnknownEither(ErrorResponse)(error);

			// Assert.
			expect(result._tag).toBe("Right");
		});
	});
});
