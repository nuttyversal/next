{ pkgs, ... }:

{
	services = {
		nzbget = {
			enable = true;

			settings = {
				MainDir = "/nas/shared/Downloads";
			};
		};
	};
}
