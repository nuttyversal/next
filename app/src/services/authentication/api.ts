import { Effect, Schema } from "effect";

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

		return Effect.tryPromise(async () => {
			const response = await fetch(endpoint, {
				method: "POST",
				body: JSON.stringify(request),
			});

			return Schema.decodeUnknownSync(RegisterResponse)(response.json());
		});
	}

	/**
	 * Login a navigator.
	 */
	login(request: LoginRequest) {
		const endpoint = `${this.baseApiUrl}/navigator/login`;

		return Effect.tryPromise(async () => {
			const response = await fetch(endpoint, {
				method: "POST",
				body: JSON.stringify(request),
			});

			return Schema.decodeUnknownSync(LoginResponse)(response.json());
		});
	}

	/**
	 * Logout a navigator.
	 */
	logout() {
		const endpoint = `${this.baseApiUrl}/navigator/logout`;

		return Effect.tryPromise(async () => {
			await fetch(endpoint, {
				method: "POST",
			});
		});
	}

	/**
	 * Get information about the logged-in navigator.
	 */
	me() {
		const endpoint = `${this.baseApiUrl}/navigator/me`;

		return Effect.tryPromise(async () => {
			const response = await fetch(endpoint, {
				method: "GET",
			});

			return Schema.decodeUnknownSync(MeResponse)(response.json());
		});
	}
}

export { AuthenticationApi };
