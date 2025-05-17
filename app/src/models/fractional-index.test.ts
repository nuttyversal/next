import { Either } from "effect";
import fc from "fast-check";

import { FractionalIndex } from "./fractional-index.ts";

/**
 * Generate a valid fractional index string.
 */
function validIndexArbitrary() {
	return fc
		.array(
			fc.integer({ min: 33, max: 126 }).map((i) => String.fromCharCode(i)),
			{ minLength: 1, maxLength: 10 },
		)
		.map((chars) => chars.join(""));
}

describe("FractionalIndex", () => {
	test("can create start and end indices", () => {
		const start = FractionalIndex.start();
		const end = FractionalIndex.end();

		expect(start.asString()).toBe("!");
		expect(end.asString()).toBe("~");
		expect(start.lessThan(end)).toBe(true);
	});

	test("can create an index between two indices", () => {
		const start = FractionalIndex.start();
		const end = FractionalIndex.end();

		// Generate an index between start and end.
		const middleResult = FractionalIndex.between(start, end);
		expect(Either.isRight(middleResult)).toBe(true);

		if (Either.isRight(middleResult)) {
			const middle = middleResult.right;
			expect(start.lessThan(middle)).toBe(true);
			expect(middle.lessThan(end)).toBe(true);

			// Generate another index between start and middle.
			const quarterResult = FractionalIndex.between(start, middle);
			expect(Either.isRight(quarterResult)).toBe(true);

			if (Either.isRight(quarterResult)) {
				const quarter = quarterResult.right;
				expect(start.lessThan(quarter)).toBe(true);
				expect(quarter.lessThan(middle)).toBe(true);
			}

			// Generate another index between middle and end.
			const threeQuartersResult = FractionalIndex.between(middle, end);
			expect(Either.isRight(threeQuartersResult)).toBe(true);

			if (Either.isRight(threeQuartersResult)) {
				const threeQuarters = threeQuartersResult.right;
				expect(middle.lessThan(threeQuarters)).toBe(true);
				expect(threeQuarters.lessThan(end)).toBe(true);
			}
		}
	});

	test("returns error for identical indices", () => {
		const index = FractionalIndex.start();
		const result = FractionalIndex.between(index, index);

		expect(Either.isLeft(result)).toBe(true);
	});

	test("rejects invalid characters", () => {
		const invalidIndices = [
			"\x00", // Null character.
			"\x1F", // Unit separator.
			" ", // Space.
			"\x7F", // Delete.
			"é", // Non-ASCII.
		];

		for (const index of invalidIndices) {
			const result = FractionalIndex.fromString(index);
			expect(Either.isLeft(result)).toBe(true);
		}
	});

	// Property-based tests.
	test("ordering is consistent", () => {
		fc.assert(
			fc.property(
				validIndexArbitrary(),
				validIndexArbitrary(),
				validIndexArbitrary(),
				(a, b, c) => {
					const aResult = FractionalIndex.fromString(a);
					const bResult = FractionalIndex.fromString(b);
					const cResult = FractionalIndex.fromString(c);

					if (
						Either.isRight(aResult) &&
						Either.isRight(bResult) &&
						Either.isRight(cResult)
					) {
						const aIndex = aResult.right;
						const bIndex = bResult.right;
						const cIndex = cResult.right;

						// If a < b and b < c, then a < c.
						if (aIndex.lessThan(bIndex) && bIndex.lessThan(cIndex)) {
							expect(aIndex.lessThan(cIndex)).toBe(true);
						}
					}

					return true;
				},
			),
		);
	});

	test("between respects ordering", () => {
		fc.assert(
			fc.property(validIndexArbitrary(), validIndexArbitrary(), (a, b) => {
				const aResult = FractionalIndex.fromString(a);
				const bResult = FractionalIndex.fromString(b);

				if (Either.isRight(aResult) && Either.isRight(bResult)) {
					const aIndex = aResult.right;
					const bIndex = bResult.right;

					// Skip if a = b, since between would fail.
					if (!aIndex.equals(bIndex)) {
						// Make sure we have a consistent ordering.
						const before = aIndex.lessThan(bIndex) ? aIndex : bIndex;
						const after = aIndex.lessThan(bIndex) ? bIndex : aIndex;
						const betweenResult = FractionalIndex.between(before, after);

						if (Either.isRight(betweenResult)) {
							const between = betweenResult.right;
							return before.lessThan(between) && between.lessThan(after);
						}
					}
				}

				return true;
			}),
		);
	});
});
