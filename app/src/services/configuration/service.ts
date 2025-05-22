import { Context, Effect, Layer } from "effect";

/**
 * A service that provides access to application configuration.
 */
class ConfigurationService extends Context.Tag("ConfigurationService")<
	ConfigurationService,
	{
		/**
		 * Gets the configuration for the application.
		 */
		readonly getConfiguration: Effect.Effect<{
			/**
			 * The base URL for the API (e.g., `https://nuttyver.se/api`).
			 */
			readonly apiBaseUrl: string;
		}>;
	}
>() {}

const ConfigurationLive = Layer.succeed(ConfigurationService, {
	getConfiguration: Effect.succeed({
		apiBaseUrl: "https://nuttyver.se/api",
	}),
});

const ConfigurationLocal = Layer.succeed(ConfigurationService, {
	getConfiguration: Effect.succeed({
		apiBaseUrl: "https://local.nuttyver.se/api",
	}),
});

export { ConfigurationLive, ConfigurationLocal, ConfigurationService };
