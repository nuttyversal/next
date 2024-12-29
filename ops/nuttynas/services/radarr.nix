{ pkgs, ... }:

{
	services = {
		radarr = {
			enable = true;
			openFirewall = true;
			dataDir = "/data/radarr";
			group = "family";
		};
	};
}
