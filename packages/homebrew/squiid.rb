class Squiid < Formula
  desc "Do advanced algebraic and RPN calculations"
  homepage "https://imaginaryinfinity.net/projects/squiid/"
  url "https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/archive/${VERSION}/squiid-${VERSION}.tar.gz"
  sha256 "${SHA256SUM}"
  license "GPL-3.0-or-later"

${BOTTLE}

  depends_on "rust" => :build
  depends_on "nng"

  def install
    # Avoid vendoring `nng`.
    # "build-nng" is the `nng` crate's only default feature. To check:
    # https://gitlab.com/neachdainn/nng-rs/-/blob/v#{nng_crate_version}/Cargo.toml
    inreplace "Cargo.toml",
              /^nng = "(.+)"$/,
              'nng = { version = "\\1", default-features = false }'
    inreplace "squiid-engine/Cargo.toml",
              /^nng = { version = "(.+)", optional = true }$/,
              'nng = { version = "\\1", optional = true, default-features = false }'

    system "cargo", "install", *std_cargo_args
  end

  def check_binary_linkage(binary, library)
    binary.dynamically_linked_libraries.any? do |dll|
      next false unless dll.start_with?(HOMEBREW_PREFIX.to_s)

      File.realpath(dll) == File.realpath(library)
    end
  end

  test do
    # squiid is a TUI app
    assert_match version.to_s, shell_output("#{bin}/squiid --version")

    assert check_binary_linkage(bin/"squiid", Formula["nng"].opt_lib/shared_library("libnng")),
      "No linkage with libnng! Cargo is likely using a vendored version."
  end
end
