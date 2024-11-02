{
	description = "Nutty's tools for music management";

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
			{
				devShells = {
					default = pkgs.mkShell {
						buildInputs = [
							nixfmtty.packages.${system}.default
							pkgs.chromaprint
							pkgs.coreutils
							pkgs.fish
							pkgs.flac
							pkgs.just
							pkgs.mp3val
							pkgs.nix
							pkgs.python3
							pkgs.python3Packages.pip
						];
					};
				};
			}
		);
}
