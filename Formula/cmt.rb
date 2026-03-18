class Cmt < Formula
  desc "Zero-dependency conventional commits CLI — prompt, linter, git hook installer"
  homepage "https://github.com/mihai-ro/cmt"
  url "https://github.com/mihai-ro/cmt/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_TAGGING"
  license "MIT"
  version "1.0.0"

  def install
    bin.install "cmt"
    (share/"cmt").install "schema/cmt.schema.json"
  end

  test do
    assert_match "cmt version #{version}", shell_output("#{bin}/cmt version")
    assert_match "feat", shell_output("#{bin}/cmt types")
    pipe_output("#{bin}/cmt lint", "feat: add login\n", 0)
    pipe_output("#{bin}/cmt lint", "bad message\n", 1)
  end
end
