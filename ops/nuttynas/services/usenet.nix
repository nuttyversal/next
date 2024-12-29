{ pkgs, ... }:

{
	services = {
		nzbget = {
			enable = true;
			stateDir = "/data/nzbget";

			settings = {
				MainDir = "/nas/shared/Downloads";
			};
		};
	};
}
