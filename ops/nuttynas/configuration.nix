{
	pkgs,
	...
}:

{
	imports = [
		# System Configuration
		./system/hardware.nix
		./system/networking.nix
		./system/users.nix

		# Service Configuration
		./services/jellyfin.nix
		./services/radarr.nix
		./services/sonarr.nix
		./services/usenet.nix
	];

	time = {
		timeZone = "America/Phoenix";
	};

	nix = {
		package = pkgs.nixVersions.stable;
		extraOptions = "experimental-features = nix-command flakes";
	};

	nixpkgs = {
		hostPlatform = "x86_64-linux";

		config = {
			allowUnfree = true;
		};
	};

	# This looks like something that should be updated. Don't do it!
	# https://nixos.wiki/wiki/FAQ/When_do_I_update_stateVersion
	system.stateVersion = "24.05";
}
