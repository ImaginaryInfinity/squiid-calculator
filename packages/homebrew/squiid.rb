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

  def read_stdout(stdout)
    output = ""
    loop do
      # strip off some color and style escape codes
      output += stdout.read_nonblock(1024).gsub(/\e\[[0-9;]+[A-Za-z]/, "")
    rescue IO::WaitReadable
      break if stdout.wait_readable(2).nil?

      retry
    rescue EOFError
      break
    end

    output
  end

  test do
    require "pty"

    PTY.spawn("squiid") do |r, w, pid|
      sleep 1 # wait for squiid to start

      w.write "(10 - 2) * (3 + 5) / 4\r"
      assert_match "(10-2)*(3+5)/4=16", read_stdout(r)
      w.write "quit\r"

      # for some reason the test hangs on macOS until stdout is read again
      read_stdout(r) unless OS.linux?
      Process.wait(pid)
    rescue PTY::ChildExited
      # app exited
    end
  end
end
