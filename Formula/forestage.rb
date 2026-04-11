# Homebrew formula for forestage (stable channel)
# Updated automatically by CI on tagged releases (v*)
# macOS only (arm64). Linux users: use install.sh or build from source.

class Forestage < Formula
  desc "Opinionated wrapper for Claude Code with persona theming"
  homepage "https://github.com/arcavenae/forestage"
  url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-darwin-arm64"
  version "VERSION_PLACEHOLDER"
  sha256 "SHA256_ARM64_PLACEHOLDER"
  license "MIT"

  def install
    bin.install "forestage-darwin-arm64" => "forestage"
  end

  def caveats
    <<~EOS
      forestage requires Claude Code CLI (claude) to be installed.
      See: https://docs.anthropic.com/en/docs/claude-code
    EOS
  end

  test do
    assert_match "forestage", shell_output("#{bin}/forestage --version 2>&1")
  end
end
