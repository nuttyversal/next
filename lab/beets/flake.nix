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
						buildInputs = with pkgs; [
							nixfmtty.packages.${system}.default
							chromaprint
							coreutils
							fish
							flac
							mp3val
							python3
							python3Packages.pip
						];
					};
				};
			}
		);
}
