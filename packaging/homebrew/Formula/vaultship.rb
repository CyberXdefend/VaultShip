class Vaultship < Formula
  desc "Encrypt, harden, bind, and sign container artifacts"
  homepage "https://github.com/cyberxdefend/vaultship"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cyberxdefend/vaultship/releases/download/v#{version}/vaultship-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    else
      url "https://github.com/cyberxdefend/vaultship/releases/download/v#{version}/vaultship-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/cyberxdefend/vaultship/releases/download/v#{version}/vaultship-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    else
      url "https://github.com/cyberxdefend/vaultship/releases/download/v#{version}/vaultship-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  def install
    bin.install "vaultship"
  end

  test do
    assert_match "vaultship", shell_output("#{bin}/vaultship --help")
  end
end
