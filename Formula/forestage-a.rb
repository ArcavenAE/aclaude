# Homebrew formula for forestage-a (alpha channel)
# Updated automatically by CI on every push to main
# macOS only (arm64). Linux users: use install.sh or build from source.

class ForestageA < Formula
  desc "Opinionated wrapper for Claude Code with persona theming (alpha channel)"
  homepage "https://github.com/arcavenae/forestage"
  url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-a-darwin-arm64"
  version "VERSION_PLACEHOLDER"
  sha256 "SHA256_ARM64_PLACEHOLDER"
  license "MIT"

  def install
    bin.install "forestage-a-darwin-arm64" => "forestage-a"
  end

  def caveats
    <<~EOS
      forestage-a is the alpha channel. Updates on every push to main.
      For stable: brew install arcavenae/tap/forestage
      Requires Claude Code CLI (claude).
    EOS
  end

  test do
    assert_match "forestage", shell_output("#{bin}/forestage-a --version 2>&1")
  end
end
