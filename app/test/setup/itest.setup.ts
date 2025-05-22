import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

/**
 * Global setup for integration tests.
 * This runs once before all integration tests.
 */
export async function setup() {
	console.log("Setting up integration test environment…\n");
}

/**
 * Global teardown for integration tests.
 * This runs once after all integration tests.
 */
export async function teardown() {
	console.log("Cleaning up integration test environment…");

	try {
		// E.g., await execAsync("npm run db:cleanup:test");
		console.log("Test cleanup completed!");
	} catch (error) {
		console.warn("Cleanup warning:", error);
	}
}
