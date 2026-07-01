class Cmt < Formula
  desc "Conventional Commits CLI — a single native binary"
  homepage "https://github.com/mihai-ro/cmt"
  version "2.0.8"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.8/cmt-aarch64-apple-darwin"
      sha256 "636b773a0825bc67042209c1a554e2ea24534ba7b32de39a872e627c75357a2e"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.8/cmt-x86_64-apple-darwin"
      sha256 "48f160d8b7061cab2395771a3adadd516f8d172516e2ef9c146ba716719f1eab"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.8/cmt-aarch64-unknown-linux-gnu"
      sha256 "f8e7ecb8df5ce795b3f7d5916e359e6333367e73ea43a0f5ce5060da0692be2a"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.8/cmt-x86_64-unknown-linux-gnu"
      sha256 "eb6071c1bd3e6385c73a1f2673fb7d7551cdc05acf3ad779e38121e4d305c829"
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
