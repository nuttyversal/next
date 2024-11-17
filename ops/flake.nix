{
	description = "Nuttyverse Ops";

	inputs = {
		nixpkgs = {
			url = "github:NixOS/nixpkgs/nixos-unstable";
		};
	};

	outputs =
		inputs@{ self, nixpkgs, ... }:
		{
			nixosConfigurations.nuttybox = nixpkgs.lib.nixosSystem {
				system = "x86_64-linux";

				modules = [
					./nuttybox/configuration.nix
					./nuttybox/configuration.hardware.nix
				];

				specialArgs = {
					inherit inputs;
				};
			};
		};
}
