class Cmt < Formula
  desc "Conventional Commits CLI — a single native binary"
  homepage "https://github.com/mihai-ro/cmt"
  version "2.0.5"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.5/cmt-aarch64-apple-darwin"
      sha256 "7b219da98414cdb76a38d7c4f89b672ffbfd3cc4d31556eaf278f859793916f2"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.5/cmt-x86_64-apple-darwin"
      sha256 "b4bae7d98685a7a10373d034f28e7ea2feef41a493820b710a3b427a3359d2f4"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.5/cmt-aarch64-unknown-linux-gnu"
      sha256 "42ffd72e7a556d0326dc5e788683e3c7cce0caaaf3152b5f5292daae226613ef"
    end
    on_intel do
      url "https://github.com/mihai-ro/cmt/releases/download/%40mihairo/cmt-v2.0.5/cmt-x86_64-unknown-linux-gnu"
      sha256 "2fa89b6353a0bd13852444b9835d9c859b1d922ebaa605b92c42daed3209d99d"
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
