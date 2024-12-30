{ pkgs, ... }:

{
	services = {
		radarr = {
			enable = true;
			openFirewall = true;
			dataDir = "/data/sonarr";
			group = "family";
		};
	};
}
