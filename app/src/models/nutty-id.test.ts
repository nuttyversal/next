import { Either, Schema } from "effect";
import fc from "fast-check";
import { Temporal } from "temporal-polyfill";

import {
	decodeBase58,
	encodeBase58,
	NuttyId,
	NuttyIdFromString,
} from "./nutty-id.ts";

describe("Base58", () => {
	test("encoding bigint to string", () => {
		// A known bigint to string conversion result.
		// See the homework assignment in nutty_id.rs.
		const bi = 1852570767862n;
		const str = encodeBase58(bi, 1);

		expect(Either.isRight(str)).toBe(true);

		if (Either.isRight(str)) {
			expect(str.right).toBe("qfWLRgy");
		}
	});

	test("decoding string to bigint", () => {
		// A known string to bigint conversion result.
		// Using the same values from the encoding test in reverse.
		const str = "qfWLRgy";
		const result = decodeBase58(str);

		expect(Either.isRight(result)).toBe(true);

		if (Either.isRight(result)) {
			expect(result.right).toBe(1852570767862n);
		}
	});

	test("roundtrip encoding and decoding", () => {
		fc.assert(
			fc.property(fc.bigInt({ min: 0n }), (input) => {
				const encoded = encodeBase58(input, 1);
				const decoded = Either.flatMap(encoded, (encoded) => {
					return decodeBase58(encoded);
				});

				return Either.match(decoded, {
					onLeft: () => false,
					onRight: (decoded) => decoded === input,
				});
			}),
		);
	});
});

describe("StringFromNuttyId", () => {
	test("decode StringifiedNuttyId to NuttyId", () => {
		const stringifiedNuttyId = "1CNjZEV7a6mVR14vf8UtLA:jzBBXYW";

		const result = // Decode the stringified ID.
			Schema.decodeUnknownEither(NuttyIdFromString)(stringifiedNuttyId);

		expect(Either.isRight(result)).toBe(true);

		if (Either.isRight(result)) {
			const nuttyId = result.right;

			// Check that we got a proper NuttyId object.
			expect(nuttyId).toHaveProperty("uuid");
			expect(nuttyId).toHaveProperty("nid");
			expect(nuttyId).toHaveProperty("timestamp");

			// Validate the nid part matches what we expect.
			expect(nuttyId.nid).toBe("jzBBXYW");

			// Validate the timestamp is a Temporal.ZonedDateTime.
			expect(nuttyId.timestamp).toBeInstanceOf(Temporal.ZonedDateTime);
		}
	});

	test("encode NuttyId to StringifiedNuttyId", () => {
		const nuttyId = NuttyId.fromUuid("0196934a-2c78-7e03-884f-bd7d01cb50ab");
		expect(Either.isRight(nuttyId)).toBe(true);

		if (Either.isRight(nuttyId)) {
			// Encode into stringified ID.
			const result = Schema.encodeUnknownEither(NuttyIdFromString)(
				nuttyId.right,
			);

			// Did the encoding succeed?
			expect(Either.isRight(result)).toBe(true);

			// Check the encoding result.
			if (Either.isRight(result)) {
				const stringifiedNuttyId = result.right;
				expect(stringifiedNuttyId).toBe("1CNjZEV7a6mVR14vf8UtLA:jzBBXYW");
			}
		}
	});
});
