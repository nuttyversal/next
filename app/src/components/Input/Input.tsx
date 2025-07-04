import { Component, JSX, splitProps } from "solid-js";

import { cm } from "~/utilities/class-merging.ts";

interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
	class?: string;
}

const Input: Component<InputProps> = (props) => {
	const [local, others] = splitProps(props, ["class"]);

	return (
		<input
			class={cm(
				"border-input file:text-foreground placeholder:text-muted-foreground focus-visible:ring-ring flex h-9 w-full rounded-md border bg-transparent px-3 py-1 text-base transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
				local.class,
			)}
			{...others}
		/>
	);
};

export { Input };
