import { createContext, ParentComponent, useContext } from "solid-js";

import {
	NuttyverseLiveRuntime,
	NuttyverseLocalRuntime,
	NuttyverseRuntime,
} from "./runtime.ts";

/**
 * The type of the runtime context. This context provides a managed runtime
 * to components in the application. The runtime is used to access services
 * and satisfy dependency requirements in effectful operations.
 */
interface RuntimeContextType {
	NuttyverseRuntime: NuttyverseRuntime;
}

/**
 * Provides a managed runtime to components in the application.
 */
const RuntimeContext = createContext<RuntimeContextType>();

/**
 * A provider component that wraps the application and provides
 * a production-level managed runtime to components.
 */
const RuntimeLiveProvider: ParentComponent = (props) => {
	const services = {
		NuttyverseRuntime: NuttyverseLiveRuntime,
	};

	return (
		<RuntimeContext.Provider value={services}>
			{props.children}
		</RuntimeContext.Provider>
	);
};

/**
 * A provider component that wraps the application and provides
 * a development-level managed runtime to components.
 */
const RuntimeLocalProvider: ParentComponent = (props) => {
	const services = {
		NuttyverseRuntime: NuttyverseLocalRuntime,
	};

	return (
		<RuntimeContext.Provider value={services}>
			{props.children}
		</RuntimeContext.Provider>
	);
};

/**
 * A hook that provides access to the runtime context.
 */
const useRuntime = () => {
	const runtime = useContext(RuntimeContext);

	if (!runtime) {
		throw new Error("Runtime context is not available.");
	}

	return runtime.NuttyverseRuntime;
};

export {
	RuntimeContext,
	RuntimeLiveProvider,
	RuntimeLocalProvider,
	useRuntime,
};
