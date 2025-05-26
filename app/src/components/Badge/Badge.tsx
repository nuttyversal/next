import { type JSX, splitProps } from "solid-js";

import { cm } from "~/utilities/class-merging.ts";
import {
	cva,
	type VariantProps,
} from "~/utilities/class-variance-authority.ts";

const badgeVariants = cva(
	"inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
	{
		variants: {
			variant: {
				default:
					"border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80",
				secondary:
					"border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
				destructive:
					"border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80",
				outline: "text-foreground",
				trace: "text-gray-500 font-mono",
			},
		},
		defaultVariants: {
			variant: "default",
		},
	},
);

export interface BadgeProps
	extends JSX.HTMLAttributes<HTMLDivElement>,
		VariantProps<typeof badgeVariants> {}

function Badge(props: BadgeProps) {
	const [local, others] = splitProps(props, ["class", "variant"]);

	return (
		<div
			class={cm(badgeVariants({ variant: local.variant }), local.class)}
			{...others}
		/>
	);
}

export { Badge, badgeVariants };
