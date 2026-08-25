# Homebrew formula for crux.
#
# Tap usage (after pushing this repo):
#   brew tap Emran-goat/crux https://github.com/Emran-goat/crux
#   brew install crux
#
# Or build straight from this file:
#   brew install --build-from-source packaging/homebrew/crux.rb

class Crux < Formula
  desc "Finds the exact commit behind a behavior change"
  homepage "https://github.com/Emran-goat/crux"
  url "https://github.com/Emran-goat/crux/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/crux --help")
  end
end
