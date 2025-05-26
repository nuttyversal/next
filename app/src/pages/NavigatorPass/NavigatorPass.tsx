import { Effect } from "effect";
import { createSignal } from "solid-js";

import { Button } from "~/components/Button/index.ts";
import { Input } from "~/components/Input/index.ts";
import { PrettyError } from "~/components/PrettyError/PrettyError.tsx";
import {
	AuthenticationService,
	authenticationStore,
} from "~/services/authentication/index.ts";
import { useRuntime } from "~/services/context.tsx";

const NavigatorPass = () => {
	const NuttyverseRuntime = useRuntime();
	const [name, setName] = createSignal("");
	const [pass, setPass] = createSignal("");

	const handleLogin = () => {
		return NuttyverseRuntime.runPromise(
			Effect.gen(function* () {
				const authService = yield* AuthenticationService;

				return yield* authService
					.login({ name: name(), pass: pass() })
					.pipe(
						Effect.tap(() => {
							setName("");
							setPass("");
						}),
						Effect.ignore,
					);
			}),
		);
	};

	const handleLogout = () => {
		return NuttyverseRuntime.runPromise(
			Effect.gen(function* () {
				const authService = yield* AuthenticationService;
				return yield* authService.logout;
			}),
		);
	};

	return (
		<div class="mx-auto h-full w-full max-w-md rounded-lg bg-white p-8 shadow-md">
			{authenticationStore.navigator ? (
				<form class="space-y-3">
					<div class="space-y-1">
						You are logged in as{" "}
						<code class="text-sm">
							{authenticationStore.navigator.name}
						</code>
						.
					</div>

					<Button
						disabled={authenticationStore.isLoading}
						onClick={handleLogout}
					>
						Logout
					</Button>
				</form>
			) : (
				<form class="space-y-3">
					<div class="space-y-1">
						<label for="name" class="text-sm font-medium text-gray-700">
							Name
						</label>

						<Input
							id="name"
							type="name"
							value={name()}
							onInput={(e) => setName(e.target.value.toLowerCase())}
							style={{ "font-variant-caps": "small-caps" }}
							class="w-full"
						/>
					</div>

					<div class="space-y-1">
						<label for="pass" class="text-sm font-medium text-gray-700">
							Pass
						</label>

						<Input
							id="pass"
							type="password"
							value={pass()}
							onInput={(e) => setPass(e.target.value)}
							class="w-full"
						/>
					</div>

					<Button
						class="space-y-1"
						disabled={authenticationStore.isLoading}
						onClick={handleLogin}
					>
						Login
					</Button>

					{authenticationStore.error && (
						<PrettyError error={authenticationStore.error} />
					)}
				</form>
			)}
		</div>
	);
};

export { NavigatorPass };
