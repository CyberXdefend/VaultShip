{
  description = "VaultShip — encrypt, harden, bind, and sign container artifacts";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages = rec {
          vaultship = pkgs.rustPlatform.buildRustPackage {
            pname = "vaultship";
            version =
              (builtins.fromTOML
                (builtins.readFile ./crates/vaultship-cli/Cargo.toml))
              .package.version;
            src = pkgs.lib.cleanSource self;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "--package" "vaultship-cli" ];
            meta = with pkgs.lib; {
              description = "Encrypt, harden, bind, and sign container artifacts";
              homepage = "https://github.com/cyberxdefend/vaultship";
              license = licenses.asl20;
              mainProgram = "vaultship";
            };
          };
          default = vaultship;
        };

        apps = rec {
          vaultship = {
            type = "app";
            program = "${self.packages.${system}.vaultship}/bin/vaultship";
          };
          default = vaultship;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            clippy
            rust-analyzer
            cargo-watch
          ];
        };
      }
    );
}
