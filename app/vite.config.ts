import tailwindcss from "@tailwindcss/vite";
import { visualizer } from "rollup-plugin-visualizer";
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
	plugins: [
		solidPlugin(),
		tailwindcss(),
		visualizer({
			open: true,
			gzipSize: true,
		}),
	],
	server: {
		port: 3000,
	},
	build: {
		target: "esnext",
	},
});
