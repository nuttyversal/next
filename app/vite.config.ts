import tailwindcss from "@tailwindcss/vite";
import { visualizer } from "rollup-plugin-visualizer";
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
	plugins: [
		solidPlugin(),
		tailwindcss(),
		tsconfigPaths(),
		visualizer({
			open: true,
			gzipSize: true,
		}),
	],
	server: {
		allowedHosts: ["local.nuttyver.se"],
		host: "0.0.0.0",
		port: 3000,
	},
	build: {
		target: "esnext",
	},
});
