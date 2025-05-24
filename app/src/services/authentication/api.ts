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
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(request),
				credentials: "include",
			});

			const responseJson = await response.json();

			return Schema.decodeUnknownSync(RegisterResponse)(responseJson);
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
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify(request),
				credentials: "include",
			});

			const responseJson = await response.json();

			return Schema.decodeUnknownSync(LoginResponse)(responseJson);
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
				credentials: "include",
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
				credentials: "include",
			});

			const responseJson = await response.json();

			return Schema.decodeUnknownSync(MeResponse)(responseJson);
		});
	}
}

export { AuthenticationApi };
