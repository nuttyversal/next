{ pkgs, ... }:

{
	services = {
		jellyfin = {
			enable = true;
			openFirewall = true;
			dataDir = "/data/jellyfin";
			configDir = "/data/jellyfin/config";
		};
	};

	environment = {
		systemPackages = [
			pkgs.jellyfin
			pkgs.jellyfin-web
			pkgs.jellyfin-ffmpeg
		];
	};
}
