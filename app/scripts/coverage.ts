import { execSync } from "child_process";
import { cp } from "fs/promises";

const main = async () => {
	const coverageDirectory = "coverage";
	const utestDirectory = "utest";
	const itestDirectory = "itest";

	const coverageFiles = [utestDirectory, itestDirectory].map(
		(testDirectory) => {
			return {
				testType: testDirectory,
				filename: `${coverageDirectory}/${testDirectory}/coverage-final.json`,
			};
		},
	);

	await Promise.all(
		coverageFiles.map((coverageFile) => {
			return cp(
				coverageFile.filename,
				`${coverageDirectory}/combined/${coverageFile.testType}.json`,
			);
		}),
	);

	console.log("Creating test coverage report…\n");

	// prettier-ignore
	execSync("pnpm exec nyc merge coverage/combined coverage/coverage-final.json");

	// prettier-ignore
	execSync("pnpm exec nyc report -t coverage --report-dir coverage/html --reporter=html-spa");

	// prettier-ignore
	execSync("pnpm exec nyc report -t coverage", { stdio: "inherit" });
};

void main();
