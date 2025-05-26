import { createStore } from "solid-js/store";

import { Navigator } from "~/models/navigator.ts";
import { PrettyError } from "~/models/pretty-error.ts";

const [authenticationStore, setAuthenticationStore] = createStore({
	navigator: null as Navigator | null,
	error: null as PrettyError | null,
	isLoading: false,
});

export { authenticationStore, setAuthenticationStore };
