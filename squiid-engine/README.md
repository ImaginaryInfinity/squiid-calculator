# Compiling
To compile the shared object file, just run install Rust and run `cargo build --release`. The shared object file should be found in `target/release/libsquiid_engine.so`.

To compile the release binary (if you cannot use the shared object file from your program), run `cargo build --release --bin squiid_engine_bin`. The release binary should be found in `target/release/squiid_engine_bin`.