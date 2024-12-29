{ pkgs, ... }:

{
	services = {
		radaar = {
			enable = true;
			openFirewall = true;
			dataDir = "/data/radaar";
		};
	};
}
