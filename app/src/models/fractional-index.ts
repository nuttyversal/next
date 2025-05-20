import { Either, ParseResult, Predicate, Schema } from "effect";

// Exclamation mark (!)
const MIN_CHAR = 33;

// Tilde (~)
const MAX_CHAR = 126;

// Number of possible values per digit.
const BASE = 94;

/**
 * Validate if a string contains only valid base-94 characters.
 */
function isValidFractionalIndex(index: string): boolean {
	return index.split("").every((c) => {
		const code = c.charCodeAt(0);
		return code >= MIN_CHAR && code <= MAX_CHAR;
	});
}

/**
 * A fractional index for ordering content blocks.
 *
 * The index is stored as a base-94 string, where each character represents
 * a digit in the range [33, 126] (the set of visible ASCII characters),
 * which enables generation of new index between any two existing indices
 * by averaging their values together.
 */
class FractionalIndex extends Schema.Class<FractionalIndex>("FractionalIndex")({
	index: Schema.String.pipe(
		Schema.filter(isValidFractionalIndex, {
			message: (index) => `Invalid character in index: '${index}'`,
		}),
	),
}) {
	/**
	 * Creates a new `FractionalIndex` from a `string`.
	 */
	static fromString(index: string): Either.Either<FractionalIndex, string> {
		if (!isValidFractionalIndex(index)) {
			const invalidChar = index
				.split("")
				.find(Predicate.not(isValidFractionalIndex));

			return Either.left(`Invalid character: ${invalidChar}`);
		}

		return Either.right(
			new FractionalIndex({
				index,
			}),
		);
	}

	/**
	 * Creates a new `FractionalIndex` at the start of the sequence.
	 */
	static start(): FractionalIndex {
		return new FractionalIndex({
			index: "!",
		});
	}

	/**
	 * Creates a new `FractionalIndex` at the end of the sequence.
	 */
	static end(): FractionalIndex {
		return new FractionalIndex({
			index: "~",
		});
	}

	/**
	 * Generates a new index between two existing indices.
	 */
	static between(
		before: FractionalIndex,
		after: FractionalIndex,
	): Either.Either<FractionalIndex, string> {
		const beforeIndex = before.asString();
		const afterIndex = after.asString();

		// Check for identical indices.
		if (beforeIndex === afterIndex) {
			return Either.left(
				`Identical indices: ${beforeIndex} === ${afterIndex}`,
			);
		}

		let result = "";
		let carry = 0;

		// Pad the shorter string with minimum value characters.
		const maxLen = Math.max(beforeIndex.length, afterIndex.length);
		const beforePadded = beforeIndex.padEnd(maxLen, "!");
		const afterPadded = afterIndex.padEnd(maxLen, "!");

		// Process each character position.
		for (let i = 0; i < maxLen; i++) {
			// Convert characters to numeric values: [33, 126] ↦ [0, 93].
			const bVal = beforePadded.charCodeAt(i) - MIN_CHAR;
			const aVal = afterPadded.charCodeAt(i) - MIN_CHAR;

			// Calculate the sum and carry.
			const sum = bVal + aVal + carry;
			const digit = Math.floor(sum / 2);
			carry = (sum % 2) * BASE;

			// Convert digit back to character: [0, 93] ↦ [33, 126].
			const c = String.fromCharCode(digit + MIN_CHAR);
			result += c;
		}

		// If there's a carry, add an additional digit.
		if (carry > 0) {
			const c = String.fromCharCode(Math.floor(carry / 2) + MIN_CHAR);
			result += c;
		}

		return FractionalIndex.fromString(result);
	}

	/**
	 * Returns the string representation of the index.
	 */
	asString(): string {
		return this.index;
	}

	/**
	 * Compare this index with another index.
	 */
	compare(other: FractionalIndex): number {
		// Pad the shorter string with minimum value characters.
		const maxLen = Math.max(this.index.length, other.index.length);
		const selfPadded = this.index.padEnd(maxLen, "!");
		const otherPadded = other.index.padEnd(maxLen, "!");

		if (selfPadded < otherPadded) return -1;
		if (selfPadded > otherPadded) return 1;
		return 0;
	}

	/**
	 * Check if this index is less than another index.
	 */
	lessThan(other: FractionalIndex): boolean {
		return this.compare(other) < 0;
	}

	/**
	 * Check if this index is greater than another index.
	 */
	greaterThan(other: FractionalIndex): boolean {
		return this.compare(other) > 0;
	}

	/**
	 * Check if this index is equal to another index.
	 */
	equals(other: FractionalIndex): boolean {
		return this.compare(other) === 0;
	}
}

/**
 * Schema transformation between `FractionalIndex` and `string`.
 */
const FractionalIndexFromString = Schema.transformOrFail(
	Schema.String,
	FractionalIndex,
	{
		strict: true,
		decode: (input, _options, ast) => {
			if (!isValidFractionalIndex(input)) {
				const invalidChar = input
					.split("")
					.find(Predicate.not(isValidFractionalIndex));

				return ParseResult.fail(
					new ParseResult.Type(
						ast,
						input,
						`Invalid character: ${invalidChar}`,
					),
				);
			}

			return ParseResult.succeed(
				new FractionalIndex({
					index: input,
				}),
			);
		},
		encode: (input, _option, _ast) => {
			return ParseResult.succeed(input.index);
		},
	},
);

export { FractionalIndex, FractionalIndexFromString };
