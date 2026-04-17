# Homebrew formula for forestage (stable channel)
# Updated automatically by CI on tagged releases (v*)
# macOS (arm64) and Linux (amd64, arm64) supported.

class Forestage < Formula
  desc "Opinionated wrapper for Claude Code with persona theming"
  homepage "https://github.com/arcavenae/forestage"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-darwin-arm64"
    sha256 "SHA256_DARWIN_ARM64_PLACEHOLDER"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-linux-arm64"
    sha256 "SHA256_LINUX_ARM64_PLACEHOLDER"
  elsif OS.linux?
    url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-linux-amd64"
    sha256 "SHA256_LINUX_AMD64_PLACEHOLDER"
  end

  def install
    if OS.mac? && Hardware::CPU.arm?
      bin.install "forestage-darwin-arm64" => "forestage"
    elsif OS.linux? && Hardware::CPU.arm?
      bin.install "forestage-linux-arm64" => "forestage"
    elsif OS.linux?
      bin.install "forestage-linux-amd64" => "forestage"
    end
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
