{
  description = "Universal Package Manager";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f rec {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
            lib = pkgs.lib;
            craneLib = crane.mkLib pkgs;
          }
        );
    in
    {
      packages = forAllSystems (
        {
          pkgs,
          lib,
          craneLib,
          ...
        }:
        let
          unipac = craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;
          };
          plugins = [ "pacman" ];

          mkPlugin =
            name:
            pkgs.stdenvNoCC.mkDerivation {
              pname = "unipac-plugin-${name}";
              version = unipac.version;
              src = ./plugins/${name};
              buildPhase = ''
                mkdir -p $out/bin
                cp -r ${name}.sh $out/bin/unipac-plugin-${name}
              '';
              strictDeps = true;
              buildInputs = with pkgs; [ bash ];
            };
          pluginsPkgs = lib.genAttrs plugins mkPlugin;
        in
        {
          inherit unipac;
          default = unipac;
          unipac-plugin-pacman = mkPlugin "pacman";
        }
        // pluginsPkgs
      );

      devShells = forAllSystems (
        { craneLib, ... }:
        {
          default = craneLib.devShell {
            shellHook = ''
              export PATH="$PATH:$PWD/unipac-shared"
            '';
          };
        }
      );
    };
}
