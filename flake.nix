{
  description = "Keeps your default applications the way you set them";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forEachSystem = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      overlays.default = final: _prev: {
        mime-guardian = final.callPackage ./nix/package.nix { };
      };

      packages = forEachSystem (
        system: rec {
          mime-guardian = (pkgsFor system).callPackage ./nix/package.nix { };
          default = mime-guardian;
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.mime-guardian ];

            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.rustfmt
              pkgs.rust-analyzer
              pkgs.cocogitto
              pkgs.shellcheck
            ];
          };
        }
      );

      formatter = forEachSystem (system: (pkgsFor system).nixfmt-rfc-style);
    };
}
