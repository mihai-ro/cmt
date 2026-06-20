class Cmt < Formula
  desc "Conventional Commits CLI — a single native binary"
  homepage "https://github.com/mihai-ro/cmt"
  version "2.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-aarch64-apple-darwin"
      sha256 "REPLACE_WITH_ARM64_DARWIN_SHA"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-x86_64-apple-darwin"
      sha256 "REPLACE_WITH_X64_DARWIN_SHA"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-aarch64-unknown-linux-gnu"
      sha256 "REPLACE_WITH_ARM64_LINUX_SHA"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/v#{version}/cmt-x86_64-unknown-linux-gnu"
      sha256 "REPLACE_WITH_X64_LINUX_SHA"
    end
  end

  def install
    bin.install Dir["cmt-*"].first => "cmt"
  end

  test do
    assert_match "cmt version", shell_output("#{bin}/cmt --version")
  end
end
