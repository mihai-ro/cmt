class Cmt < Formula
  desc "Conventional Commits CLI — a single native binary"
  homepage "https://github.com/mihai-ro/cmt"
  version "2.0.6"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.6/cmt-aarch64-apple-darwin"
      sha256 "dd9bfffbad5916c0b7a6f0b163a9a378b0332219ac2cdf6f456dfa8d27b971a7"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.6/cmt-x86_64-apple-darwin"
      sha256 "feb5909c8482ce7bdfa302b598b642dccf35bca373d280a5821dd02dc9fd4e95"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.6/cmt-aarch64-unknown-linux-gnu"
      sha256 "6242506b84842aa0992abe5f785508539ba7fe7d526f10b4672dace19eaaffdd"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.6/cmt-x86_64-unknown-linux-gnu"
      sha256 "6cbb9d9276e1bbf557863c777322e45871144e4f5a8dd565511a9c9358dea7c6"
    end
  end

  def install
    bin.install Dir["cmt-*"].first => "cmt"
  end

  test do
    assert_match "cmt version #{version}", shell_output("#{bin}/cmt --version")
    assert_match "feat", shell_output("#{bin}/cmt types")
    pipe_output("#{bin}/cmt lint", "feat: add login\n", 0)
    pipe_output("#{bin}/cmt lint", "bad message\n", 1)
  end
end
