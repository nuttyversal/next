{
	description = "Nuttyverse Ops";

	inputs = {
		nixpkgs = {
			url = "github:NixOS/nixpkgs/nixos-unstable";
		};

		agenix = {
			url = "github:ryantm/agenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};

		flake-utils = {
			url = "github:numtide/flake-utils";
		};
	};

	outputs =
		inputs@{
			self,
			nixpkgs,
			agenix,
			flake-utils,
			...
		}:
		{
			nixosConfigurations = {
				nuttybox = nixpkgs.lib.nixosSystem {
					system = "x86_64-linux";

					modules = [
						./nuttybox/configuration.nix
						agenix.nixosModules.default
					];

					specialArgs = {
						inherit inputs;
					};
				};

				nuttygate = nixpkgs.lib.nixosSystem {
					system = "x86_64-linux";

					modules = [
						./nuttygate/configuration.nix
					];

					specialArgs = {
						inherit inputs;
					};
				};
			};
		}
		// flake-utils.lib.eachDefaultSystem (
			system:
			let
				pkgs = import nixpkgs {
					inherit system;
				};
			in
			{
				devShells.default = pkgs.mkShell {
					buildInputs = [
						agenix.packages.${system}.default
					];
				};
			}
		);
}
