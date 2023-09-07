class Squiid < Formula
  desc "Modular calculator written in Rust"
  homepage "https://imaginaryinfinity.net/"
  url "https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/archive/${VERSION}/squiid-${VERSION}.tar.gz"
  sha256 "${SHA256SUM}"
  license "GPL-3.0-only"

  depends_on "cmake" => :build
  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/squiid", "--version"
  end
end
