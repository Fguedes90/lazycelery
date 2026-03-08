class Lazycelery < Formula
  desc "A terminal UI for monitoring and managing Celery workers and tasks, inspired by lazydocker/lazygit"
  homepage "https://github.com/Fguedes90/lazycelery"
  version "VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Fguedes90/lazycelery/releases/download/vVERSION/lazycelery-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256"
    else
      url "https://github.com/Fguedes90/lazycelery/releases/download/vVERSION/lazycelery-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256"
    end
  end

  on_linux do
    url "https://github.com/Fguedes90/lazycelery/releases/download/vVERSION/lazycelery-linux-x86_64.tar.gz"
    sha256 "PLACEHOLDER_SHA256"
  end

  def install
    bin.install "lazycelery"
  end

  test do
    assert_match "lazycelery", shell_output("#{bin}/lazycelery --help")
  end
end
