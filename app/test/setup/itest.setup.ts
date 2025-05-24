import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

/**
 * Global setup for integration tests.
 * This runs once before all integration tests.
 */
export async function setup() {
	console.log("Setting up integration test environment…");

	// A self-signed certificate is being used for local.nuttyver.se.
	console.log("Disabling TLS certificate verification…\n");
	process.env.NODE_TLS_REJECT_UNAUTHORIZED = "0";
}

/**
 * Global teardown for integration tests.
 * This runs once after all integration tests.
 */
export async function teardown() {
	console.log("Cleaning up integration test environment…");

	try {
		// E.g., await execAsync("npm run db:cleanup:test");
	} catch (error) {
		console.warn("Cleanup warning:", error);
	}
}
