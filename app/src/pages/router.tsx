import { Route, Router } from "@solidjs/router";

import { NavigatorPass } from "./NavigatorPass/index.ts";
import { Singularity } from "./Singularity/index.ts";

const NuttyverseRouter = () => {
	return (
		<Router>
			<Route path="/navpass" component={NavigatorPass} />
			<Route path="/snglrty" component={Singularity} />
		</Router>
	);
};

export { NuttyverseRouter };
