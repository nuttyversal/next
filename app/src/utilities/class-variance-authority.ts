/**
 * Copyright 2022 Joe Bell. All rights reserved.
 *
 * This file is licensed to you under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with the
 * License. You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */

import type * as CLSX from "./class-conditional.ts";
import { clsx } from "./class-conditional.ts";

type ClassValue = CLSX.ClassValue;

type ClassProp =
	| {
			class: ClassValue;
			className?: never;
	  }
	| { class?: never; className: ClassValue }
	| { class?: never; className?: never };

type OmitUndefined<T> = T extends undefined ? never : T;
type StringToBoolean<T> = T extends "true" | "false" ? boolean : T;

type VariantProps<Component extends (...args: any) => any> = Omit<
	OmitUndefined<Parameters<Component>[0]>,
	"class" | "className"
>;

const falsyToString = <T extends unknown>(value: T) =>
	typeof value === "boolean" ? `${value}` : value === 0 ? "0" : value;

const cx = clsx;

type ConfigSchema = Record<string, Record<string, ClassValue>>;

type ConfigVariants<T extends ConfigSchema> = {
	[Variant in keyof T]?: StringToBoolean<keyof T[Variant]> | null | undefined;
};

type ConfigVariantsMulti<T extends ConfigSchema> = {
	[Variant in keyof T]?:
		| StringToBoolean<keyof T[Variant]>
		| StringToBoolean<keyof T[Variant]>[]
		| undefined;
};

type Config<T> = T extends ConfigSchema
	? {
			variants?: T;
			defaultVariants?: ConfigVariants<T>;
			compoundVariants?: (T extends ConfigSchema
				? (ConfigVariants<T> | ConfigVariantsMulti<T>) & ClassProp
				: ClassProp)[];
		}
	: never;

type Props<T> = T extends ConfigSchema
	? ConfigVariants<T> & ClassProp
	: ClassProp;

const cva = <T>(base?: ClassValue, config?: Config<T>) => {
	return (props?: Props<T>) => {
		if (config?.variants == null)
			return cx(base, props?.class, props?.className);

		const { variants, defaultVariants } = config;

		const getVariantClassNames = Object.keys(variants).map(
			(variant: keyof typeof variants) => {
				const variantProp = props?.[variant as keyof typeof props];
				const defaultVariantProp = defaultVariants?.[variant];

				if (variantProp === null) return null;

				const variantKey = (falsyToString(variantProp) ||
					falsyToString(
						defaultVariantProp,
					)) as keyof (typeof variants)[typeof variant];

				return variants[variant][variantKey];
			},
		);

		const propsWithoutUndefined =
			props &&
			Object.entries(props).reduce(
				(acc, [key, value]) => {
					if (value === undefined) {
						return acc;
					}

					acc[key] = value;
					return acc;
				},
				{} as Record<string, unknown>,
			);

		const getCompoundVariantClassNames = config?.compoundVariants?.reduce(
			(
				acc,
				{
					class: cvClass,
					className: cvClassName,
					...compoundVariantOptions
				},
			) =>
				Object.entries(compoundVariantOptions).every(([key, value]) =>
					Array.isArray(value)
						? value.includes(
								{
									...defaultVariants,
									...propsWithoutUndefined,
								}[key],
							)
						: {
								...defaultVariants,
								...propsWithoutUndefined,
							}[key] === value,
				)
					? [...acc, cvClass, cvClassName]
					: acc,
			[] as ClassValue[],
		);

		return cx(
			base,
			getVariantClassNames,
			getCompoundVariantClassNames,
			props?.class,
			props?.className,
		);
	};
};

export { cva, type VariantProps };
