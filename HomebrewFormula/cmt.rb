class Cmt < Formula
  desc "Conventional Commits CLI — a single native binary"
  homepage "https://github.com/mihai-ro/cmt"
  version "2.0.7"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.7/cmt-aarch64-apple-darwin"
      sha256 "70b93adfd6b6a7e686bf1d2c4af990177a18754773476a689067cf675323535c"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.7/cmt-x86_64-apple-darwin"
      sha256 "12bfd54bc975f2b5591941e05e0cd1b8aa8ba6c71b1b1726fae8fb8d46ac4e21"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.7/cmt-aarch64-unknown-linux-gnu"
      sha256 "c36a1d495d5831a6d07fc18e6cbc85de61b649a9a1112647099102a0401cab8c"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.7/cmt-x86_64-unknown-linux-gnu"
      sha256 "1c4f3776213075c2660bf8d729a2bf4cb983426a7190f8dce6febd680522425d"
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
