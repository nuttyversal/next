{
  inputs = { nixpkgs.url = "github:nixos/nixpkgs"; flake-utils.url = "github:numtide/flake-utils"; };
  outputs = { self, flake-utils, nixpkgs, }:
    { overlay = import ./overlay.nix; }
    // flake-utils.lib.eachDefaultSystem (
      system: let pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.hello = pkgs.hello;
        defaultPackage = self.packages.${system}.hello;
        devShells.hello = pkgs.mkShell {buildInputs = [pkgs.hello pkgs.cowsay];};
        devShell = self.devShells.${system}.hello;
      }
    );
}
