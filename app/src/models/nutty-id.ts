import { Either, ParseResult, Schema } from "effect";
import { UUID } from "effect/Schema";
import { Temporal } from "temporal-polyfill";
import { v7 as uuidv7 } from "uuid";

// Base-58 alphabet (₿-encoding).
const BASE_58_ALPHABET =
	"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/**
 * Encode a number to base-58 with padding.
 */
function encodeBase58(
	value: bigint,
	padWidth: number,
): Either.Either<string, string> {
	if (value < 0n) {
		return Either.left("Invalid input: value must be positive");
	}

	if (value === 0n) {
		return Either.right("1".repeat(padWidth));
	}

	const base = 58n;
	const digits: number[] = [];
	let remaining = value;

	while (remaining > 0n) {
		digits.push(Number(remaining % base));
		remaining = remaining / base;
	}

	// Pad with "1"s if needed.
	const paddingLength = Math.max(0, padWidth - digits.length);
	let result = "1".repeat(paddingLength);

	// Encode digits in big-endian order.
	for (let i = digits.length - 1; i >= 0; i--) {
		result += BASE_58_ALPHABET[digits[i]];
	}

	return Either.right(result);
}

/**
 * Decode a base-58 string to a number.
 */
function decodeBase58(input: string): Either.Either<bigint, string> {
	if (input.length === 0) {
		return Either.left("Invalid input: empty string");
	}

	const base = 58n;
	let result = 0n;

	for (const char of input) {
		const index = BASE_58_ALPHABET.indexOf(char);

		if (index === -1) {
			return Either.left(`Invalid character '${char}' in base58 string`);
		}

		result = result * base + BigInt(index);
	}

	return Either.right(result);
}

/**
 * Extract the last 41 bits from a UUID string.
 */
function extractLast41Bits(uuid: string): bigint {
	// Remove hyphens and convert to hex.
	const hex = uuid.replace(/-/g, "");

	// Convert the last 6 bytes (12 hex chars) to a number.
	const lastBytes = hex.slice(-12);
	const value = BigInt("0x" + lastBytes);

	// Mask to get only the last 41 bits.
	const mask = (1n << 41n) - 1n;
	return value & mask;
}

/**
 * Extract timestamp from UUIDv7 (first 48 bits).
 */
function extractTimestamp(uuid: string): number {
	const hex = uuid.replace(/-/g, "");
	const timestampHex = hex.slice(0, 12); // First 6 bytes
	return Number(BigInt("0x" + timestampHex));
}

/**
 * Validate if a string is a valid Nutty ID (7 characters, base58, <= "zmM9z4E").
 */
const isValidNuttyId = (id: string): boolean => {
	// It must be exactly 7 characters.
	if (id.length !== 7) {
		return false;
	}

	// It must contain only valid characters.
	if (!id.split("").every((c) => BASE_58_ALPHABET.includes(c))) {
		return false;
	}

	// It must be derived from the last 41 bits of a UUID.
	return id <= "zmM9z4E";
};

/**
 * Schema for a dissociated Nutty ID (just the 7-character NID).
 *
 * This poor Nutty ID is traumatized.
 *
 * Lost in the wild, with no UUID to call its own,
 * it wanders through memory, a nomad without a home.
 *
 * Any Nutty ID can be derived from a UUID,
 * but the path backward is darkened, you see.
 *
 * To find its ancestral UUID, you must decree
 * a solemn query to the content block tree.
 *
 * tl;dr: it's surjective, but not injective.
 */
const DissociatedNuttyId = Schema.String.pipe(
	Schema.filter(isValidNuttyId, {
		message: (nid) => `Invalid Nutty ID format: '${nid}'`,
	}),
	Schema.brand("DissociatedNuttyId"),
);

type DissociatedNuttyId = typeof DissociatedNuttyId.Type;

/**
 * Schema for the serialized format of a NuttyId: "BASE58_UUID:NID".
 */
const StringifiedNuttyId = Schema.String.pipe(
	Schema.pattern(/^[1-9A-HJ-NP-Za-km-z]{22}:[1-9A-HJ-NP-Za-km-z]{7}$/),
	Schema.brand("StringifiedNuttyId"),
);

type StringifiedNuttyId = typeof StringifiedNuttyId.Type;

/**
 * Schema for the deserialized format of a NuttyId.
 */
class NuttyId extends Schema.Class<NuttyId>("NuttyId")({
	uuid: UUID,
	nid: DissociatedNuttyId,
	timestamp: Schema.instanceOf(Temporal.ZonedDateTime),
}) {
	/**
	 * Create a NuttyId from a newly generated UUIDv7.
	 */
	static now(): NuttyId {
		const uuid = uuidv7();
		const nuttyId = NuttyId.fromUuid(uuid);

		if (Either.isLeft(nuttyId)) {
			throw new Error("Assertion: NuttyId.fromUuid failed with valid UUID");
		}

		return nuttyId.right;
	}

	/**
	 * Create a NuttyId from an existing UUIDv7.
	 */
	static fromUuid(uuid: typeof UUID.Type): Either.Either<NuttyId, string> {
		// Extract the last 41 bits and encode as base58 to get the NID.
		const last41Bits = extractLast41Bits(uuid);
		const encodedNid = encodeBase58(last41Bits, 7);

		if (Either.isLeft(encodedNid)) {
			return Either.left(encodedNid.left);
		}

		// Extract timestamp and convert to ZonedDateTime.
		const timestampMs = extractTimestamp(uuid);
		const instant = Temporal.Instant.fromEpochMilliseconds(timestampMs);
		const timeZoneId = Temporal.Now.timeZoneId();
		const timestamp = instant.toZonedDateTimeISO(timeZoneId);

		return Either.right(
			new NuttyId({
				uuid,
				nid: encodedNid.right as DissociatedNuttyId,
				timestamp,
			}),
		);
	}
}

/**
 * Schema transformation between `NuttyId` and `StringifiedNuttyId`.
 */
const NuttyIdFromString = Schema.transformOrFail(StringifiedNuttyId, NuttyId, {
	strict: true,
	decode: (input, _options, ast) => {
		const [uuidBase58, nid] = input.split(":");

		// Decode the UUID from base58.
		const uuidResult = decodeBase58(uuidBase58);

		if (Either.isLeft(uuidResult)) {
			return ParseResult.fail(
				new ParseResult.Type(ast, input, "Invalid UUID encoding"),
			);
		}

		// Convert the bigint back to UUID format.
		const uuidHex = uuidResult.right.toString(16).padStart(32, "0");
		const uuid = UUID.make(
			[
				uuidHex.slice(0, 8),
				uuidHex.slice(8, 12),
				uuidHex.slice(12, 16),
				uuidHex.slice(16, 20),
				uuidHex.slice(20, 32),
			].join("-"),
		);

		// Calculate what the NID should be from the UUID.
		const last41Bits = extractLast41Bits(uuid);
		const calculatedNid = encodeBase58(last41Bits, 7);

		if (Either.isLeft(calculatedNid)) {
			return ParseResult.fail(
				new ParseResult.Type(
					ast,
					input,
					`Failed to encode NID: ${calculatedNid.left}`,
				),
			);
		}

		// Verify the NID matches (checksum validation).
		if (calculatedNid.right !== nid) {
			return ParseResult.fail(
				new ParseResult.Type(
					ast,
					input,
					`NID mismatch: expected ${calculatedNid}, got ${nid}`,
				),
			);
		}

		// Extract timestamp (assuming UUIDv7).
		const timestampMs = extractTimestamp(uuid);
		const instant = Temporal.Instant.fromEpochMilliseconds(timestampMs);
		const timeZoneId = Temporal.Now.timeZoneId();
		const timestamp = instant.toZonedDateTimeISO(timeZoneId);

		return ParseResult.succeed({
			uuid,
			nid: nid as DissociatedNuttyId,
			timestamp,
		});
	},
	encode: (input, _options, ast) => {
		// Convert UUID to bigint.
		const uuidHex = input.uuid.replace(/-/g, "");
		const uuidBigInt = BigInt("0x" + uuidHex);

		// Encode to base-58.
		const uuidBase58 = encodeBase58(uuidBigInt, 22);

		if (Either.isLeft(uuidBase58)) {
			return ParseResult.fail(
				new ParseResult.Type(ast, input, "Failed to base-58 encode UUID"),
			);
		}

		// Brand the result.
		const stringifiedNuttyId = StringifiedNuttyId.make(
			`${uuidBase58.right}:${input.nid}`,
		);

		return ParseResult.succeed(stringifiedNuttyId);
	},
}).pipe(
	Schema.annotations({
		identifier: "NuttyId",
		title: "Nutty ID",
		description: "A nutty wrapper around a UUID",
	}),
);

export {
	decodeBase58,
	DissociatedNuttyId,
	encodeBase58,
	NuttyId,
	NuttyIdFromString,
	StringifiedNuttyId,
};
