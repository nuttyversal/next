{
	description = "Nutty's privacy guard tools";

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
							pkgs.coreutils
							pkgs.gnupg
						];
					};
				};
			}
		);
}
