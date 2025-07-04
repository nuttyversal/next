import { render } from "@solidjs/testing-library";
import { Effect } from "effect";
import { beforeEach, describe, expect, it } from "vitest";

import { getRootElement, main } from "./index.tsx";
import { NuttyverseTestRuntime } from "./services/runtime.ts";

describe("Application startup", () => {
	it("renders without crashing", async () => {
		render(() => (
			<div id="root">
				<div id="loading" />
			</div>
		));

		await NuttyverseTestRuntime.runPromise(main);
	});

	describe("getRootElement", () => {
		beforeEach(() => {
			document.body.innerHTML = "";
		});

		it("returns the root element", async () => {
			render(() => <div id="root"></div>);
			const rootElement = await Effect.runPromise(getRootElement);
			expect(rootElement).not.toBe(null);
		});

		it("throws an error if the root element is not found", async () => {
			// Arrange.
			const rootElementOrError = Effect.runSync(
				getRootElement.pipe(Effect.catchAll(Effect.succeed)),
			);

			// Assert.
			expect(rootElementOrError).toBeInstanceOf(Error);
		});
	});
});
