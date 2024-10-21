{
	description = "Nutty's Neovim";

	inputs = {
		nixpkgs = {
			url = "github:NixOS/nixpkgs/nixos-unstable";
		};

		flake-utils = {
			url = "github:numtide/flake-utils";
		};

		nixfmtty = {
			url = "github:nuttyversal/next?dir=nix/pkgs/nixfmtty";
		};
	};

	outputs =
		{
			self,
			flake-utils,
			nixpkgs,
			nixfmtty,
		}:
		flake-utils.lib.eachDefaultSystem (
			system:
			let
				pkgs = import nixpkgs {
					inherit system;
				};
			in
			rec {
				packages = {
					neovim = pkgs.neovim;
				};

				devShells.default = pkgs.mkShell {
					buildInputs = pkgs.lib.attrsets.attrValues packages ++ [
						nixfmtty.packages.${system}.default
					];
				};
			}
		);
}
