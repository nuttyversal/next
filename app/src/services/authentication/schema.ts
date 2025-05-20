import { Schema } from "effect";

import { NavigatorFromApi, NavigatorName } from "~/models/navigator.ts";
import { SessionFromApi } from "~/models/session.ts";

/**
 * Request payload for registering a new navigator.
 */
const RegisterRequest = Schema.Struct({
	name: NavigatorName,
	pass: Schema.String.pipe(Schema.minLength(8), Schema.maxLength(128)),
});

type RegisterRequest = typeof RegisterRequest.Type;

/**
 * Request payload for logging in a navigator.
 */
const LoginRequest = Schema.Struct({
	name: Schema.String,
	pass: Schema.String,
});

type LoginRequest = typeof LoginRequest.Type;

/**
 * Login response model containing both navigator and session.
 */
const LoginResponse = Schema.Struct({
	navigator: NavigatorFromApi,
	session: SessionFromApi,
});

type LoginResponse = typeof LoginResponse.Type;

export { LoginRequest, LoginResponse, RegisterRequest };
