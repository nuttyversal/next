{
	description = "Nutty's tools for Lua development";

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
							pkgs.lua
							pkgs.lua-language-server
							pkgs.lua54Packages.luarocks
							pkgs.stylua
						];
					};
				};
			}
		);
}
