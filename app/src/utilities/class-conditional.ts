/**
 * MIT License
 *
 * Copyright (c) Luke Edwards <luke.edwards05@gmail.com> (lukeed.com)
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

type ClassValue =
	| ClassArray
	| ClassDictionary
	| string
	| number
	| bigint
	| null
	| boolean
	| undefined;

type ClassDictionary = Record<string, any>;
type ClassArray = ClassValue[];

function toVal(mix: ClassValue): string {
	let k: number;
	let y: string;
	let str = "";

	if (typeof mix === "string" || typeof mix === "number") {
		str += mix;
	} else if (typeof mix === "object" && mix !== null) {
		if (Array.isArray(mix)) {
			const len = mix.length;
			for (k = 0; k < len; k++) {
				if (mix[k]) {
					y = toVal(mix[k]);
					if (y) {
						if (str) str += " ";
						str += y;
					}
				}
			}
		} else {
			for (y in mix) {
				if (mix[y]) {
					if (str) str += " ";
					str += y;
				}
			}
		}
	}

	return str;
}

/**
 * Conditionally join class names together.
 *
 * @param inputs - Any number of class values to conditionally join
 * @returns A space-separated string of class names
 *
 * @example
 * ```ts
 * clsx('foo', 'bar'); // 'foo bar'
 * clsx('foo', { bar: true, baz: false }); // 'foo bar'
 * clsx(['foo', 'bar']); // 'foo bar'
 * clsx('foo', null, undefined, 0, 1, { bar: true }); // 'foo 1 bar'
 * ```
 */
function clsx(...inputs: ClassValue[]): string {
	let i = 0;
	let tmp: ClassValue;
	let x: string;
	let str = "";
	const len = inputs.length;

	for (; i < len; i++) {
		tmp = inputs[i];
		if (tmp) {
			x = toVal(tmp);
			if (x) {
				if (str) str += " ";
				str += x;
			}
		}
	}

	return str;
}

export { type ClassValue, clsx };
