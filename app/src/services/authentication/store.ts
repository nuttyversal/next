import { createStore } from "solid-js/store";

import { ApiError } from "~/models/api.ts";
import { Navigator } from "~/models/navigator.ts";

const [authenticationStore, setAuthenticationStore] = createStore({
	loggedInNavigator: null as Navigator | null,
	isLoggingIn: false,
	errors: [] as readonly ApiError[],
});

export { authenticationStore, setAuthenticationStore };
