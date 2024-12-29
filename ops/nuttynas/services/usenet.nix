{ pkgs, ... }:

{
	services = {
		nzbget = {
			enable = true;
			group = "family";

			settings = {
				MainDir = "/nas/shared/Downloads";
			};
		};
	};
}
