{
  description =
    "An implementation of the SMP protocol as used in zephyr, mcuboot, mcumgr, and more.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "/nixpkgs";
      inputs.nixpkgs-stable.follows = "/nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, pre-commit }:
    {
      overlays.smp-rs = final: prev: {
        smp-rs = final.callPackage
          ({ lib, stdenv, rustPlatform, pkg-config, udev, libiconv, darwin }:
            rustPlatform.buildRustPackage {
              pname = "smp-rs";
              version =
                self.shortRev or "dirty-${toString self.lastModifiedDate}";
              src = self;
              cargoLock = {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };

              nativeBuildInputs = lib.optional stdenv.isLinux pkg-config;
              buildInputs = (lib.optional stdenv.isLinux udev)
                ++ lib.optional stdenv.isDarwin [
                  libiconv
                  darwin.apple_sdk.frameworks.IOKit
                ];

              meta = {
                description =
                  "An implementation of the SMP protocol as used in zephyr, mcuboot, mcumgr, and more.";
                homepage = "https://github.com/Gessler-GmbH/smp-rs";
                license = with lib.licenses; [ asl20 mit ];
                platforms = lib.platforms.unix;
                mainProgram = "smp-tool";
              };
            }) { };
      };
      overlays.default = self.overlays.smp-rs;
    } // flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      in {
        packages = {
          inherit (pkgs) smp-rs;
          default = pkgs.smp-rs;
        };
        legacyPackages = pkgs;

        devShells = {
          smp-rs = pkgs.mkShell {
            nativeBuildInputs = with pkgs;
              [ rust-analyzer nixfmt rustfmt ] ++ pkgs.smp-rs.nativeBuildInputs;
            buildInputs = pkgs.smp-rs.buildInputs;

            inherit (self.checks.${system}.pre-commit) shellHook;
          };
          default = self.devShells.${system}.smp-rs;
        };

        formatter = pkgs.nixfmt;

        checks = {
          pre-commit = pre-commit.lib.${system}.run {
            src = self;
            hooks = {
              rustfmt.enable = true;
              nixfmt.enable = true;
            };
          };
        };
      });
}
