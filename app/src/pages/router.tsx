import { Route, Router } from "@solidjs/router";

import Singularity from "./Singularity/index.ts";

const NuttyverseRouter = () => {
	return (
		<Router>
			<Route path="/" component={Singularity} />
		</Router>
	);
};

export { NuttyverseRouter };
