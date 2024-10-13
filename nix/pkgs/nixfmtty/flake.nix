{
	description = "Nutty's nixfmt wrapper";

	inputs = {
		nixpkgs = {
			url = "github:NixOS/nixpkgs/nixos-unstable";
		};

		flake-utils = {
			url = "github:numtide/flake-utils";
		};
	};

	outputs =
		{
			self,
			nixpkgs,
			flake-utils,
		}:
		flake-utils.lib.eachDefaultSystem (
			system:
			let
				pkgs = nixpkgs.legacyPackages.${system};

				buildInputs = [
					pkgs.fish
					pkgs.moreutils
					pkgs.nixfmt-rfc-style
				];

				nixfmtty = pkgs.stdenv.mkDerivation {
					name = "nixfmtty";
					src = ./.;

					nativeBuildInputs = [
						pkgs.makeWrapper
					];

					buildInputs = buildInputs;

					installPhase =
						let
							path = pkgs.lib.makeBinPath buildInputs;
						in
						''
							mkdir -p $out/bin
							cp ${./scripts/format.fish} $out/bin/nixfmtty
							chmod +x $out/bin/nixfmtty
							wrapProgram $out/bin/nixfmtty --prefix PATH : ${path}
						'';

					meta = with pkgs.lib; {
						description = "Nutty's nixfmt wrapper";
						homepage = "https://nuttyver.se";
						license = licenses.mit;
						maintainers = [ maintainers.nuttyversal ];
						platforms = platforms.all;
					};
				};
			in
			{
				packages = {
					default = nixfmtty;
				};

				devShells = {
					default = pkgs.mkShell {
						buildInputs = buildInputs ++ [
							pkgs.just
							nixfmtty
						];
					};
				};
			}
		);
}
