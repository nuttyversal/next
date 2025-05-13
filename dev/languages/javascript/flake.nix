{
	description = "Nutty's tools for JS/TS development";

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
							pkgs.nodejs
							pkgs.eslint_d
							pkgs.nodePackages.eslint
							pkgs.nodePackages.pnpm
							pkgs.nodePackages.prettier
							pkgs.nodePackages.sass
							pkgs.nodePackages.ts-node
							pkgs.nodePackages.typescript
							pkgs.nodePackages.typescript-language-server
							pkgs.tailwindcss-language-server
						];
					};
				};
			}
		);
}
