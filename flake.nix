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
          unipac-shared = buildBash {
            name = "unipac-shared";
            src = ./unipac-shared;
            buildPhase = "cp * $out/bin";
          };

          buildBash =
            {
              name,
              src,
              buildPhase,
              buildInputs ? [ ],
              args ? { },
            }:
            pkgs.stdenvNoCC.mkDerivation (
              {
                pname = name;
                inherit src;
                version = unipac.version;
                buildPhase = ''
                  mkdir -p $out/bin
                  ${buildPhase}
                '';
                strictDeps = true;
                buildInputs = [ pkgs.bash ] ++ buildInputs;
              }
              // args
            );

          mkPlugin =
            {
              name,
              buildInputs ? [ ],
            }:
            buildBash {
              name = "unipac-plugin-${name}";
              src = ./plugins/${name};
              buildPhase = "cp -r ${name}.sh $out/bin/unipac-plugin-${name}";
              args = {
                nativeBuildInputs = [ pkgs.makeWrapper ];
                installPhase = ''
                  mkdir -p $out/bin
                  wrapProgram $out/bin/unipac-plugin-${name} \
                    --prefix PATH : ${lib.makeBinPath buildInputs}
                '';
              };
              buildInputs = buildInputs ++ [ unipac-shared ];
            };
        in
        {
          inherit unipac unipac-shared;
          default = unipac;
          unipac-plugin-pacman = mkPlugin { name = "pacman"; };
          unipac-plugin-nix-profile = mkPlugin {
            name = "nix-profile";
            buildInputs = with pkgs; [
              jq
              nix-search-cli
            ];
          };
        }
      );

      devShells = forAllSystems (
        { pkgs, craneLib, ... }:
        {
          default = craneLib.devShell {
            packages = with pkgs; [
              jq
              nix-search-cli
            ];
            shellHook = ''
              export PATH="$PATH:$PWD/unipac-shared"
            '';
          };
        }
      );
    };
}
