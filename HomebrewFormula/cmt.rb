class Cmt < Formula
  desc "Conventional Commits CLI — a single native binary"
  homepage "https://github.com/mihai-ro/cmt"
  version "2.0.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v#{version}/cmt-aarch64-apple-darwin"
      sha256 "cfcfd20f8b5de87fe8cae47d2987a7b6bc8a534ae805fbca20975c1bc1e22f45"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v#{version}/cmt-x86_64-apple-darwin"
      sha256 "e48e186395955ce1509c85b01981d6e722d435cd4f0199d8e90d668eb757eb7c"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v#{version}/cmt-aarch64-unknown-linux-gnu"
      sha256 "20973e5792a86f00599ed52e5408ca1e7f2d645e9d6be979981152bcf61a8f4f"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v#{version}/cmt-x86_64-unknown-linux-gnu"
      sha256 "6f8b999dd09818e5095602a57aeedc5cf7b0a0b136b54ae8aa82f29979f1d41a"
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
