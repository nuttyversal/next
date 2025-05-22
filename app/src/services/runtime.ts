import { Layer, ManagedRuntime, pipe } from "effect";

import { AuthenticationLive } from "./authentication/index.ts";
import {
	ConfigurationLive,
	ConfigurationLocal,
} from "./configuration/index.ts";

type NuttyverseRuntime = typeof NuttyverseLiveRuntime;

const NuttyverseLiveRuntime = ManagedRuntime.make(
	pipe(AuthenticationLive, Layer.provideMerge(ConfigurationLive)),
);

const NuttyverseLocalRuntime: NuttyverseRuntime = ManagedRuntime.make(
	pipe(AuthenticationLive, Layer.provideMerge(ConfigurationLocal)),
);

export {
	NuttyverseLiveRuntime,
	NuttyverseLocalRuntime,
	type NuttyverseRuntime,
};
