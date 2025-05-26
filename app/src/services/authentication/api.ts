import { Effect, Schema } from "effect";

import { RequestError } from "~/models/api.ts";

import {
	LoginRequest,
	LoginResponse,
	MeResponse,
	RegisterRequest,
	RegisterResponse,
} from "./schema.ts";

class AuthenticationApi {
	readonly baseApiUrl: string;

	constructor(baseApiUrl: string) {
		this.baseApiUrl = baseApiUrl;
	}

	/**
	 * Register a navigator.
	 */
	register(request: RegisterRequest) {
		const endpoint = `${this.baseApiUrl}/navigator`;

		const makeRequest = Effect.tryPromise({
			try: async () => {
				const response = await fetch(endpoint, {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify(request),
					credentials: "include",
				});

				return await response.json();
			},

			catch: () => {
				return new RequestError();
			},
		});

		return Effect.andThen(
			makeRequest,
			Schema.decodeUnknown(RegisterResponse),
		);
	}

	/**
	 * Login a navigator.
	 */
	login(request: LoginRequest) {
		const endpoint = `${this.baseApiUrl}/navigator/login`;

		const makeRequest = Effect.tryPromise({
			try: async () => {
				const response = await fetch(endpoint, {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify(request),
					credentials: "include",
				});

				return await response.json();
			},

			catch: () => {
				return new RequestError();
			},
		});

		return Effect.andThen(makeRequest, Schema.decodeUnknown(LoginResponse));
	}

	/**
	 * Logout a navigator.
	 */
	logout() {
		const endpoint = `${this.baseApiUrl}/navigator/logout`;

		return Effect.tryPromise({
			try: async () => {
				const response = await fetch(endpoint, {
					method: "POST",
					credentials: "include",
				});

				return await response.json();
			},

			catch: () => {
				return new RequestError();
			},
		});
	}

	/**
	 * Get information about the logged-in navigator.
	 */
	me() {
		const endpoint = `${this.baseApiUrl}/navigator/me`;

		const makeRequest = Effect.tryPromise({
			try: async () => {
				const response = await fetch(endpoint, {
					method: "GET",
					credentials: "include",
				});

				return await response.json();
			},

			catch: () => {
				return new RequestError();
			},
		});

		return Effect.andThen(makeRequest, Schema.decodeUnknown(MeResponse));
	}
}

export { AuthenticationApi };
