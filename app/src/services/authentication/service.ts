import { Context, Effect, Layer } from "effect";
import { ParseError } from "effect/ParseResult";

import { RequestError, Response } from "~/models/api.ts";

import { ConfigurationService } from "../configuration/index.ts";
import { AuthenticationApi } from "./api.ts";
import {
	LoginRequest,
	MeResponse,
	RegisterRequest,
	RegisterResponse,
} from "./schema.ts";
import { setAuthenticationStore } from "./store.ts";

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
		) => Effect.Effect<void, RequestError | ParseError>;

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
		const authApi = new AuthenticationApi(config.apiBaseUrl);

		const register = Effect.fn(function* (request: RegisterRequest) {
			return yield* authApi.register(request);
		});

		const login = Effect.fn(function* (request: LoginRequest) {
			setAuthenticationStore("isLoading", true);
			const response = yield* authApi.login(request);

			return Response.match(response, {
				onError: (errors) => {
					setAuthenticationStore({
						isLoading: false,
						errors,
					});
				},

				onData: (data) => {
					setAuthenticationStore({
						navigator: data?.navigator ?? null,
						isLoading: false,
						errors: [],
					});
				},
			});
		});

		const logout = Effect.gen(function* () {
			setAuthenticationStore("isLoading", true);
			yield* authApi.logout();

			setAuthenticationStore({
				navigator: null,
				isLoading: false,
			});
		});

		const me = authApi.me();

		return { register, login, logout, me };
	}),
);

export { AuthenticationLive, AuthenticationService };
