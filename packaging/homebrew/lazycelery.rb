class Lazycelery < Formula
  desc "Terminal UI for monitoring and managing Celery workers and tasks"
  homepage "https://github.com/Fguedes90/lazycelery"
  url "https://github.com/Fguedes90/lazycelery/archive/v0.2.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "lazycelery", shell_output("#{bin}/lazycelery --help")
  end
end