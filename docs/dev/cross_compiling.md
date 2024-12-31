# Cross Compiling

This page contains special cross compiling instructions from Linux to other platforms. All of the commands in this documentation were tested in Arch Linux. If you have any issue building for a specific platform, please [open an issue](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/issues/) and we may be able to help.

Most cross compiling should work out of the box with [cross](https://github.com/cross-rs/cross), but if there are issues with specific build targets, they'll be listed here.

## GLIBC not found

If using `cross` to cross compile and you encounter an error similar to this:

```
/target/release/build/num-traits-7eff81b48f38ff21/build-script-build: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.29' not found (required by /target/release/build/num-traits-7eff81b48f38ff21/build-script-build)
```

It should be able to be fixed by running `cargo clean` and trying again.

