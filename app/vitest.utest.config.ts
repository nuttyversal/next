import solidPlugin from "vite-plugin-solid";
import { defineConfig } from "vitest/config";

export default defineConfig({
	plugins: [solidPlugin()],
	test: {
		testTimeout: 1000,
		environment: "jsdom",
		globals: true,
		setupFiles: "./test/setup/vitest.setup.ts",
		include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
		exclude: ["**/node_modules/*", "**/*.itest.ts"],
		coverage: {
			provider: "istanbul",
			reportsDirectory: "./coverage/utest",
			reporter: ["json", "html"],
			include: ["src/**"],
			exclude: ["**/node_modules/*", "**/*.itest.ts"],
		},
		alias: {
			"~/": new URL("./src/", import.meta.url).pathname,
		},
	},
});
