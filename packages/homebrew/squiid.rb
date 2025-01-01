class Squiid < Formula
  desc "Do advanced algebraic and RPN calculations"
  homepage "https://imaginaryinfinity.net/projects/squiid/"
  url "https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/archive/${VERSION}/squiid-${VERSION}.tar.gz"
  sha256 "${SHA256SUM}"
  license "GPL-3.0-or-later"

${BOTTLE}

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # squiid is a TUI app
    assert_match version.to_s, shell_output("#{bin}/squiid --version")
  end
end
