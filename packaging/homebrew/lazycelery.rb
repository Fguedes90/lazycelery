class Lazycelery < Formula
  desc "A terminal UI for monitoring and managing Celery workers and tasks, inspired by lazydocker/lazygit"
  homepage "https://github.com/Fguedes90/lazycelery"
  version "0.4.5"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Fguedes90/lazycelery/releases/download/v0.4.5/lazycelery-macos-aarch64.tar.gz"
      sha256 "97b1ad921373e1f8b71a309be1bc67dc47ec1a555b3c7f8ab49211e9c56b7352"
    else
      url "https://github.com/Fguedes90/lazycelery/releases/download/v0.4.5/lazycelery-macos-x86_64.tar.gz"
      sha256 "4a6fc2e614968860c259973427ff02d8431b96f091640bf1f1c146d2c2a8f726"
    end
  end

  on_linux do
    url "https://github.com/Fguedes90/lazycelery/releases/download/v0.4.5/lazycelery-linux-x86_64.tar.gz"
    sha256 "71201741d7d920ea417491bf490d4f33006e3f3da2ff9139e4f73019b6145472"
  end

  def install
    bin.install "lazycelery"
  end

  test do
    assert_match "lazycelery", shell_output("#{bin}/lazycelery --help")
  end
end