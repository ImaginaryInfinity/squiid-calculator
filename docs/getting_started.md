# Getting Started

This section features a quick introduction on how to install and use Squiid with the default Ratatui frontend.

----

## Installation

Squiid is available on numerous platforms and through many different package managers <!-- TODO: repology once we have actual packages--> (or it will be soon). Find your preferred method of installation below, or if Squiid is not available for your platform, see [building](#manual-installation) or [submitting a package request](#requesting-a-packaged-version-of-squiid).

#### Package Manager
# NOTICE: Squiid has not been packaged for package managers yet. This documentation is placeholder
The easiest way to install Squiid is through your system's package manager. A comprehensive list of available versions can be found below, along with installation instructions for some operating systems. <!-- TODO: if you'd like squiid to be packaged -->

#### Windows Users
If Windows users would like to install Squiid without the use of `winget`, we provide an installer which can be found as an exe file attached to our [latest release](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/releases/permalink/latest).

#### Manual Installation
If you would like to manually build Squiid, for example if there isn't a package for your operating system or you'd just prefer to build from source, you can follow the steps below:

1. Clone the repository with git or download the source code at [https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid).
2. Install build requirements:
    - Install Rust for your operating system. [Rustup](https://rustup.rs/) is recommended, but you can also install Rust from the [official website](https://www.rust-lang.org/).
    - Install [cmake](https://cmake.org/) for your operating system.
3. Open a terminal (or command prompt/powershell on Windows) and navigate to the source code with the `cd` command.
4. Build and install Squiid
    -  To install with make - run `sudo make install`.
    -  To install manually - run `cargo build --release`. The resulting binary can then be found at `target/release/squiid`. Copy this binary to a directory in your PATH (such as `/usr/bin`) or use the binary portably.
5. To uninstall Squiid, run `sudo make uninstall` or manually remove the binary that you placed.

#### Requesting a packaged version of Squiid
If we do not provide a packaged version of Squiid for your operating system, you can [start an issue]() to request one.