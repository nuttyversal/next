import "~/styles/global.css";

import { Effect, Option } from "effect";
import { render } from "solid-js/web";

import { NuttyverseRouter } from "~/pages/router.tsx";

import { AuthenticationService } from "./services/authentication/index.ts";
import {
	RuntimeLiveProvider,
	RuntimeLocalProvider,
} from "./services/context.tsx";
import { NuttyverseLocalRuntime } from "./services/runtime.ts";

/**
 * An effect that queries the root element of the application.
 * This is the element where the application will be rendered.
 *
 * @returns The root element of the application.
 */
const getRootElement = Effect.sync(() =>
	Option.fromNullable(document.getElementById("root")),
).pipe(
	Effect.flatMap((maybeRoot) =>
		Option.match(maybeRoot, {
			onNone: () => Effect.fail(new Error("Root element not found.")),
			onSome: Effect.succeed,
		}),
	),
);

/**
 * An effect that queries the loading element of the application.
 * This is the element that appears when fetching the application
 * bundle.
 *
 * @returns The loading element of the application.
 */
const getLoadingElement = Effect.sync(() =>
	Option.fromNullable(document.getElementById("loading")),
).pipe(
	Effect.flatMap((maybeLoading) =>
		Option.match(maybeLoading, {
			onNone: () => Effect.fail(new Error("Loading element not found.")),
			onSome: Effect.succeed,
		}),
	),
);

/**
 * An effect that hides the loading state that appears when fetching
 * the application bundle.
 */
const hideLoadingState = (loading: HTMLElement) =>
	Effect.sync(() => {
		// Trigger fade-out animation.
		loading.style.opacity = "0";

		// Wait for animation to finish before …
		loading.addEventListener("transitionend", () => {
			// … removing the element.
			loading.remove();
		});
	});

/**
 * An effect that renders the application to the root element.
 * This effect is dependent on the root element being available.
 */
const renderApplication = (root: HTMLElement) => {
	return Effect.try({
		try: () => {
			let NuttyverseRuntimeProvider;
			const hostname = window.location.hostname;
			const isTestEnvironment = import.meta.env.MODE === "test";

			if (hostname === "local.nuttyver.se" || isTestEnvironment) {
				NuttyverseRuntimeProvider = RuntimeLocalProvider;
			} else if (hostname === "nuttyver.se") {
				NuttyverseRuntimeProvider = RuntimeLiveProvider;
			} else {
				throw new Error("Nuttyverse running on unsupported domain");
			}

			return render(
				() => (
					<NuttyverseRuntimeProvider>
						<NuttyverseRouter />
					</NuttyverseRuntimeProvider>
				),
				root,
			);
		},

		catch: (error) => {
			return new Error(`Failed to render application: ${error}`);
		},
	});
};

/**
 * The main effect that serves as the entry point for the application.
 * This effect queries the root element and mounts the application to it.
 */
const main = Effect.gen(function* () {
	const root = yield* getRootElement;
	const loading = yield* getLoadingElement;
	const authService = yield* AuthenticationService;

	yield* authService.initialize;
	yield* renderApplication(root);
	yield* hideLoadingState(loading);
});

// Run the main effect and log any errors to the console.
if (import.meta.env.MODE !== "test") {
	NuttyverseLocalRuntime.runPromise(main)
		.then(() => console.log("Application started successfully!"))
		.catch((error) => console.error("Application failed to start:", error));
}

export { getRootElement, main, renderApplication };
