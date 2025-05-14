import solidPlugin from "vite-plugin-solid";
import { defineConfig } from "vitest/config";

export default defineConfig({
	plugins: [solidPlugin()],
	test: {
		environment: "jsdom",
		globals: true,
		setupFiles: "./vitest.setup.ts",
		coverage: {
			provider: "istanbul",
		},
		alias: {
			"~/": new URL("./src/", import.meta.url).pathname,
		},
	},
});
