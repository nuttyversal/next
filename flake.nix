{
	description = "Nuttyverse";

	inputs = {
		nixpkgs = {
			url = "github:NixOS/nixpkgs/nixos-unstable";
		};

		flake-utils = {
			url = "github:numtide/flake-utils";
		};

		beets = {
			url = "./lab/beets";
			inputs.nixpkgs.follows = "nixpkgs";
		};

		nixfmtty = {
			url = "./nix/pkgs/nixfmtty";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs =
		inputs@{
			self,
			nixpkgs,
			flake-utils,
			...
		}:
		flake-utils.lib.eachDefaultSystem (
			system:
			let
				pkgs = nixpkgs.legacyPackages.${system};
			in
			{
				packages = {
					inherit (inputs.nixfmtty.packages.${system}) default;
				};

				devShells = {
					default = pkgs.mkShell {
						inputsFrom = [
							inputs.beets.devShells.${system}.default
							inputs.nixfmtty.devShells.${system}.default
						];
					};
				};
			}
		);
}
