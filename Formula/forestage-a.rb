# Homebrew formula for forestage-a (alpha channel)
# Updated automatically by CI on every push to develop
# macOS (arm64) and Linux (amd64, arm64) supported.

class ForestageA < Formula
  desc "Opinionated wrapper for Claude Code with persona theming (alpha channel)"
  homepage "https://github.com/arcavenae/forestage"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-a-darwin-arm64"
    sha256 "SHA256_DARWIN_ARM64_PLACEHOLDER"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-a-linux-arm64"
    sha256 "SHA256_LINUX_ARM64_PLACEHOLDER"
  elsif OS.linux?
    url "https://github.com/arcavenae/forestage/releases/download/TAG_PLACEHOLDER/forestage-a-linux-amd64"
    sha256 "SHA256_LINUX_AMD64_PLACEHOLDER"
  end

  def install
    if OS.mac? && Hardware::CPU.arm?
      bin.install "forestage-a-darwin-arm64" => "forestage-a"
    elsif OS.linux? && Hardware::CPU.arm?
      bin.install "forestage-a-linux-arm64" => "forestage-a"
    elsif OS.linux?
      bin.install "forestage-a-linux-amd64" => "forestage-a"
    end
  end

  def caveats
    <<~EOS
      forestage-a is the alpha channel. Updates on every push to develop.
      For stable: brew install arcavenae/tap/forestage
      Requires Claude Code CLI (claude).
    EOS
  end

  test do
    assert_match "forestage", shell_output("#{bin}/forestage-a --version 2>&1")
  end
end
