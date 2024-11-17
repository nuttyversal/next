{
	inputs,
	config,
	lib,
	pkgs,
	...
}:

with lib;
{
	imports = [
		# System Configuration
		./system/networking.nix
		./system/openssh.nix

		# Service Configuration
		# …

		# Container Configuration
		# …
	];

	users = {
		users = {
			root = {
				hashedPassword = "$6$Be73trjnXBl/aQqG$bJr64Tvq8tAsONCPg9Qzc2knUBphcdf315EOpbM73chiwR4bzt4hlfOOnFxWDszqNXQzg0s27IGWBsDsiRQ1d1";
			};
		};
	};

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
