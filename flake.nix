{
	description = "Nuttyverse";

	inputs = {
		nixpkgs = {
			url = "github:NixOS/nixpkgs/nixos-unstable";
		};

		flake-utils = {
			url = "github:numtide/flake-utils";
		};

		nixfmtty = {
			url = "./nix/pkgs/nixfmtty";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs =
		{
			self,
			nixpkgs,
			flake-utils,
			nixfmtty,
		}:
		flake-utils.lib.eachDefaultSystem (
			system:
			let
				pkgs = nixpkgs.legacyPackages.${system};
			in
			{
				packages = {
					inherit (nixfmtty.packages.${system}) default;
				};

				devShells.default = pkgs.mkShell {
					buildInputs = [
						nixfmtty.packages.${system}.default
					];
				};
			}
		);
}
