# Getting Started

This section features a quick introduction on how to install and use Squiid with the default Ratatui frontend.

----

## Installation

Squiid is available on numerous platforms and through many different package managers. Find your preferred method of installation below, or if Squiid is not available for your platform, see [building](#manual-installation) or [submitting a package request](#requesting-a-packaged-version-of-squiid).

----

## Package Manager
The easiest way to install Squiid is through your system's package manager. A comprehensive list of available versions can be found below, along with installation instructions for some operating systems.

[![Packaging status](https://repology.org/badge/vertical-allrepos/squiid.svg)](https://repology.org/project/squiid/versions)

### Flatpak

```properties
flatpak install net.imaginaryinfinity.Squiid
```
<a href='https://flathub.org/apps/net.imaginaryinfinity.Squiid'><img width='180' alt='Download on Flathub' src='https://dl.flathub.org/assets/badges/flathub-badge-en.png'/></a>

### Snap
```properties
snap install squiid
```

[![Get it from the Snap Store](https://snapcraft.io/static/images/badges/en/snap-store-black.svg)](https://snapcraft.io/squiid)

### AUR

Install Squiid with your AUR helper (e.g. `paru`, `yay`):
```properties
paru -S squiid
```

Or manually:
```properties
git clone https://aur.archlinux.org/squiid.git
cd squiid
makepkg -si
```

----

## MacOS Users

### Homebrew

```properties
brew install squiid
```

<!-- ----

TODO: more -->

----

## Windows Users
If Windows users would like to install Squiid without the use of `winget`, we provide both an installer and a portable release which can be found attached to our [latest release](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/releases/permalink/latest).

### Winget
```properties
winget install --id=ImaginaryInfinity.Squiid -e
```


----

## Manual Installation
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

----

## Requesting a packaged version of Squiid
If we do not provide a packaged version of Squiid for your operating system, you can [start an issue](http://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/issues/new?issuable_template=Package%20Request) to request one.