import { Arbitrary, Effect, Either, FastCheck, Schema } from "effect";

import { ErrorResponse } from "~/models/api.ts";
import { fetchWithCookies } from "~/test/mocks/fetch.ts";

import { NuttyverseTestRuntime } from "../runtime.ts";
import { LoginRequest, RegisterRequest } from "./schema.ts";
import { AuthenticationService } from "./service.ts";

describe("AuthenticationService", () => {
	beforeAll(() => {
		// Patch the Fetch API to handle cookies in a
		// similar manner to how browsers handle cookies.
		vitest.spyOn(global, "fetch").mockImplementation(fetchWithCookies);
	});

	test("authentication flow", async () => {
		// Arrange.
		const [registerRequest] = FastCheck.sample(
			Arbitrary.make(RegisterRequest),
			1,
		);

		// Act: Register a navigator.
		const registerResponse = await NuttyverseTestRuntime.runPromise(
			Effect.gen(function* () {
				const authService = yield* AuthenticationService;
				return yield* authService.register(registerRequest);
			}),
		);

		// Assert: OK.
		expect(registerResponse).toBeDefined();

		// Act: Login as the navigator.
		const loginRequest: LoginRequest = registerRequest;

		const loginResponse = await NuttyverseTestRuntime.runPromise(
			Effect.gen(function* () {
				const authService = yield* AuthenticationService;
				return yield* authService.login(loginRequest);
			}),
		);

		// Assert: OK.
		expect(loginResponse).toBeDefined();

		// Act: Make an authenticated request to a protected endpoint.
		const meResponse = await NuttyverseTestRuntime.runPromise(
			Effect.gen(function* () {
				const authService = yield* AuthenticationService;
				return yield* authService.me;
			}),
		);

		// Assert: OK.
		expect(meResponse).toBeDefined();

		// Act: Logout the navigator.
		await NuttyverseTestRuntime.runPromise(
			Effect.gen(function* () {
				const authService = yield* AuthenticationService;
				return yield* authService.logout;
			}),
		);

		// Act: Make an unauthenticated request to a protected endpoint.
		const otherMeResponse = await NuttyverseTestRuntime.runPromise(
			Effect.gen(function* () {
				const authService = yield* AuthenticationService;
				return yield* authService.me;
			}),
		);

		// Assert: Not OK.
		const decodeErrorResponse = Schema.decodeUnknownEither(ErrorResponse);
		const maybeErrorResponse = decodeErrorResponse(otherMeResponse);
		expect(Either.isRight(maybeErrorResponse)).toBe(true);
	});
});
