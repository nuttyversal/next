import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		environment: "node",
		globals: true,
		globalSetup: ["./test/setup/itest.setup.ts"],
		setupFiles: "./test/setup/vitest.setup.ts",
		include: ["**/*.itest.ts"],
		testTimeout: 5000,

		// Path alias mappings.
		alias: {
			"~/test/": new URL("./test/", import.meta.url).pathname,
			"~/": new URL("./src/", import.meta.url).pathname,
		},

		// Run integration tests sequentially to avoid
		// race conditions between shared resources.
		pool: "forks",
		poolOptions: {
			forks: {
				singleFork: true,
			},
		},
	},
});
