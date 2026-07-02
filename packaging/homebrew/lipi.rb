# Homebrew formula for LIPI (macOS / Linux).
# Publish in a tap repo (e.g. github.com/Nemi-swami/homebrew-tap), then:
#   brew install Nemi-swami/tap/lipi
#
# Update `url`, `sha256`, and `homepage` to your release tarball before publishing.
class Lipi < Formula
  desc "Programming language with Devanagari syntax, compiled to a bytecode VM (pure Rust)"
  homepage "https://github.com/Nemi-swami/LIPI-Hindi-Programing-Lang"
  url "https://github.com/Nemi-swami/LIPI-Hindi-Programing-Lang/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "REPLACE_WITH_TARBALL_SHA256"
  license "MIT"
  head "https://github.com/Nemi-swami/LIPI-Hindi-Programing-Lang.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--locked"
    bin.install "target/release/lipi"
  end

  test do
    (testpath/"hi.swami").write('बताओ "नमस्ते"')
    assert_match "नमस्ते", shell_output("#{bin}/lipi #{testpath}/hi.swami")
  end
end
