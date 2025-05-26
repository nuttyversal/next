import { Component, For, Show } from "solid-js";

import { PrettyError as PrettyErrorType } from "~/models/pretty-error.ts";

import { Badge } from "../Badge/index.ts";

interface PrettyErrorProps {
	error: PrettyErrorType;
}

const PrettyError: Component<PrettyErrorProps> = (props) => {
	return (
		<div class="border-destructive/30 bg-destructive/5 rounded-lg border p-4">
			<h3 class="text-destructive text-base font-medium">
				{props.error.what}
			</h3>

			<p class="text-destructive/80 mt-2 text-sm">{props.error.why}</p>

			<Show when={props.error.trace.length > 0}>
				<div class="mt-4">
					<div class="flex flex-wrap items-center gap-2">
						<For each={props.error.trace}>
							{(item) => <Badge variant="trace">{item}</Badge>}
						</For>
					</div>
				</div>
			</Show>
		</div>
	);
};

export { PrettyError };
