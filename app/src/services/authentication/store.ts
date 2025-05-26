import { createStore } from "solid-js/store";

import { ApiError } from "~/models/api.ts";
import { Navigator } from "~/models/navigator.ts";

const [authenticationStore, setAuthenticationStore] = createStore({
	navigator: null as Navigator | null,
	errors: [] as readonly ApiError[],
	isLoading: false,
});

export { authenticationStore, setAuthenticationStore };
