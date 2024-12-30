{ pkgs, ... }:

{
	services = {
		sonarr = {
			enable = true;
			openFirewall = true;
			dataDir = "/data/sonarr";
			group = "family";
		};
	};
}
