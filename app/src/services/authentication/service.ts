import { Context, Effect, Layer } from "effect";
import { UnknownException } from "effect/Cause";

import { ConfigurationService } from "../configuration/index.ts";
import { AuthenticationApi } from "./api.ts";
import {
	LoginRequest,
	LoginResponse,
	MeResponse,
	RegisterRequest,
	RegisterResponse,
} from "./schema.ts";

class AuthenticationService extends Context.Tag("AuthenticationService")<
	AuthenticationService,
	{
		/**
		 * Register a navigator.
		 */
		readonly register: (
			request: RegisterRequest,
		) => Effect.Effect<RegisterResponse, UnknownException>;

		/**
		 * Login a navigator.
		 */
		readonly login: (
			request: LoginRequest,
		) => Effect.Effect<LoginResponse, UnknownException>;

		/**
		 * Logout a navigator.
		 */
		readonly logout: Effect.Effect<void, UnknownException>;

		/**
		 * Gets information about the logged-in navigator.
		 */
		readonly me: Effect.Effect<MeResponse, UnknownException>;
	}
>() {}

const AuthenticationLive = Layer.effect(
	AuthenticationService,
	Effect.gen(function* () {
		const configService = yield* ConfigurationService;
		const config = yield* configService.getConfiguration;

		const register = Effect.fn(function* (request: RegisterRequest) {
			return yield* new AuthenticationApi(config.apiBaseUrl).register(
				request,
			);
		});

		const login = Effect.fn(function* (request: LoginRequest) {
			return yield* new AuthenticationApi(config.apiBaseUrl).login(request);
		});

		const logout = new AuthenticationApi(config.apiBaseUrl).logout();

		const me = new AuthenticationApi(config.apiBaseUrl).me();

		return { register, login, logout, me };
	}),
);

export { AuthenticationLive, AuthenticationService };
