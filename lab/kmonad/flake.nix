{
	description = "KMonad advanced keyboard manager";

	inputs = {
		nixpkgs = {
			url = "github:NixOS/nixpkgs/nixos-unstable";
		};

		kmonad = {
			# This doesn't work under the `just ci` context.
			# fatal: https://github.com/kmonad/kmonad?dir=nix/info/refs not valid: is this a git repository?
			url = "git+https://github.com/kmonad/kmonad?submodules=1&dir=nix";
		};

		nixfmtty = {
			url = "github:nuttyversal/next?dir=nix/pkgs/nixfmtty";
		};
	};

	outputs =
		{
			self,
			nixpkgs,
			nixfmtty,
			kmonad,
		}:
		let
			pkgs = nixpkgs.legacyPackages.aarch64-darwin;
			kmonadPackage = kmonad.packages.aarch64-darwin.kmonad;
			config = pkgs.writeText (builtins.readFile ./macos.kbd);
		in
		rec {
			apps = {
				aarch64-darwin = {
					default = {
						type = "app";
						program = "${kmonadPackage}/bin/kmonad";
					};
				};
			};

			packages = {
				aarch64-darwin = {
					kmonad = kmonadPackage;
				};
			};

			devShells = {
				aarch64-darwin = {
					default = nixpkgs.legacyPackages.aarch64-darwin.mkShell {
						buildInputs = nixpkgs.lib.attrsets.attrValues packages.aarch64-darwin ++ [
							nixfmtty.packages.aarch64-darwin.default
							pkgs.coreutils
							pkgs.fish
							pkgs.git
							pkgs.just
							pkgs.nix
						];
					};
				};
			};
		};
}
