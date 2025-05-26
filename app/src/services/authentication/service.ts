import { Context, Data, Effect, Layer } from "effect";
import { ParseError } from "effect/ParseResult";

import { RequestError, Response } from "~/models/api.ts";
import { PrettyError } from "~/models/pretty-error.ts";

import { ConfigurationService } from "../configuration/index.ts";
import { AuthenticationApi } from "./api.ts";
import {
	LoginRequest,
	MeResponse,
	RegisterRequest,
	RegisterResponse,
} from "./schema.ts";
import { setAuthenticationStore } from "./store.ts";

/**
 * An error that is thrown when a login request fails.
 */
class LoginError extends Data.TaggedError("LoginError") {}

class AuthenticationService extends Context.Tag("AuthenticationService")<
	AuthenticationService,
	{
		/**
		 * Register a navigator.
		 */
		readonly register: (
			request: RegisterRequest,
		) => Effect.Effect<RegisterResponse, RequestError | ParseError>;

		/**
		 * Login a navigator.
		 */
		readonly login: (
			request: LoginRequest,
		) => Effect.Effect<void, LoginError>;

		/**
		 * Logout a navigator.
		 */
		readonly logout: Effect.Effect<void, RequestError>;

		/**
		 * Gets information about the logged-in navigator.
		 */
		readonly me: Effect.Effect<MeResponse, RequestError | ParseError>;
	}
>() {}

const AuthenticationLive = Layer.effect(
	AuthenticationService,
	Effect.gen(function* () {
		const configService = yield* ConfigurationService;
		const config = yield* configService.getConfiguration;
		const authenticationApi = new AuthenticationApi(config.apiBaseUrl);

		const register = Effect.fn(function* (request: RegisterRequest) {
			return yield* authenticationApi.register(request);
		});

		const login = Effect.fn(function* (request: LoginRequest) {
			setAuthenticationStore("isLoading", true);

			const response = yield* Effect.catchAll(
				authenticationApi.login(request),
				() => {
					setAuthenticationStore({
						isLoading: false,
						error: {
							what: "Login error",
							why:
								"An API request was unable to be sent to the Nuttyverse server. " +
								"Perhaps the server is down. Perhaps you are offline. Who knows? " +
								"Try again in a little bit! (･ω･)ﾉ",
							trace: [],
						},
					});

					return Effect.never;
				},
			);

			Response.match(response, {
				onData: (data) => {
					setAuthenticationStore({
						navigator: data?.navigator ?? null,
						isLoading: false,
						error: null,
					});
				},

				onError: (errors) => {
					setAuthenticationStore({
						isLoading: false,
						error: PrettyError.make({
							what: "Login error",
							why:
								"Your login request has been rejected. " +
								"Hmm… Did you type in your credentials correctly? (・_・?)",
							trace: errors.length > 0 ? errors[0].trace : [],
						}),
					});
				},
			});

			if ("errors" in response) {
				yield* new LoginError();
			}
		});

		const logout = Effect.gen(function* () {
			setAuthenticationStore("isLoading", true);
			yield* authenticationApi.logout();

			setAuthenticationStore({
				navigator: null,
				isLoading: false,
			});
		});

		const me = authenticationApi.me();

		return { register, login, logout, me };
	}),
);

export { AuthenticationLive, AuthenticationService };
