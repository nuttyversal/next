import { defineConfig } from "vite";
import { visualizer } from "rollup-plugin-visualizer";
import solidPlugin from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

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
