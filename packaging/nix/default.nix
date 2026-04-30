{ stdenv, fetchurl }:

stdenv.mkDerivation rec {
  pname = "vaultship";
  version = "0.1.0";

  src = fetchurl {
    url = "https://github.com/cyberxdefend/vaultship/releases/download/v${version}/vaultship-v${version}-x86_64-unknown-linux-gnu.tar.gz";
    sha256 = "REPLACE_WITH_SHA256";
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
}
