# Cross Compiling

This page contains special cross compiling instructions from Linux to other platforms. All of the commands in this documentation were tested in Arch Linux. If you have any issue building for a specific platform, please [open an issue](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/issues/) and we may be able to help.

## Android

## Android compilation does not work at the moment

1. Install the Android NDK. On Arch this can be installed via the [android-ndk](https://aur.archlinux.org/packages/android-ndk) AUR package. 
2. Install [cargo-ndk](https://github.com/bbqsrc/cargo-ndk) with `cargo install cargo-ndk`.
3. Install the toolchains you wish to build for:
```sh
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android

# i686-linux-android is currently unsupported due to a cross-compiling linking issue, however if anyone fixes this let us know
```

Now you can build for whatever target you'd like using the `cargo ndk` command. You must supply the `TARGET_CMAKE_TOOLCHAIN_FILE` environment variable. On my system with the AUR package installed, that would be `TARGET_CMAKE_TOOLCHAIN_FILE="/opt/android-ndk/build/cmake/android.toolchain.cmake"`. Here is an example command to build Squiid for `armv7-linux-androideabi`:

<!--TODO: change this to the makefile-->
```sh
TARGET_CMAKE_TOOLCHAIN_FILE="/opt/android-ndk/build/cmake/android.toolchain.cmake" cargo ndk --platform 33 --target armv7-linux-androideabi build --release
```

Please check the [cargo-ndk documentation](https://github.com/bbqsrc/cargo-ndk#usage) for more examples.

## AArch64 MUSL
When compiling to the target `aarch64-unknown-linux-musl` you need to use special compiler flags to get it to compile successfully. After installing the AArch64 MUSL C toolchain (gives you files like `aarch64-linux-musl-gcc`), compile the project by running:
```sh
RUSTFLAGS="-Clinker=rust-lld" cargo build --release --target=aarch64-unknown-linux-musl
```