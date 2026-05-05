{ pkgs ? import <nixpkgs> { } }:

let
  version = "0.1.0";
  sources = {
    "x86_64-linux" = {
      url = "https://github.com/cyberxdefend/vaultship/releases/download/v${version}/vaultship-v${version}-x86_64-unknown-linux-gnu.tar.gz";
      sha256 = "SHA256_X86_64_LINUX";
    };
    "aarch64-linux" = {
      url = "https://github.com/cyberxdefend/vaultship/releases/download/v${version}/vaultship-v${version}-aarch64-unknown-linux-gnu.tar.gz";
      sha256 = "SHA256_AARCH64_LINUX";
    };
    "x86_64-darwin" = {
      url = "https://github.com/cyberxdefend/vaultship/releases/download/v${version}/vaultship-v${version}-x86_64-apple-darwin.tar.gz";
      sha256 = "SHA256_X86_64_DARWIN";
    };
    "aarch64-darwin" = {
      url = "https://github.com/cyberxdefend/vaultship/releases/download/v${version}/vaultship-v${version}-aarch64-apple-darwin.tar.gz";
      sha256 = "SHA256_AARCH64_DARWIN";
    };
  };
  system = pkgs.stdenv.hostPlatform.system;
  source = sources.${system} or (throw "Unsupported system: ${system}");
in

pkgs.stdenv.mkDerivation {
  pname = "vaultship";
  inherit version;

  src = pkgs.fetchurl {
    inherit (source) url sha256;
  };

  dontUnpack = true;

  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    cp $src $TMPDIR/vaultship.tar.gz
    tar -xzf $TMPDIR/vaultship.tar.gz -C $TMPDIR
    install -m755 $TMPDIR/vaultship $out/bin/vaultship
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "Encrypt, harden, bind, and sign container artifacts";
    homepage = "https://github.com/cyberxdefend/vaultship";
    license = licenses.asl20;
    platforms = platforms.unix;
    mainProgram = "vaultship";
  };
}
