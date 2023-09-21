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
      # Wait for squiid to start
      sleep 1

      # test that math works
      w.write "(10 - 2) * (3 + 5) / 4"

      # send enter key
      w.write "\r"

      # capture the stdout into output variable
      output = read_stdout(r)

      # check that the calculator has done math correctly
      assert_match "(10-2)*(3+5)/4=16", output

      # quit
      w.write "quit"
      w.write "\r"

      # for some reason the test hangs on macOS until stdout is read again
      begin
        read_stdout(r)
      rescue Errno::EIO
        # this happens on linux but not macOS
      end

      # Wait for the TUI app to exit
      Process.wait(pid)
    rescue PTY::ChildExited
      # app exited
    end
  end
end
