# Adding an IPC Backend

Squiid Engine uses [nng-rs](https://gitlab.com/neachdainn/nng-rs) by default for IPC, but new backends can easily be added. You might want to add a new backend if, for example, the default nng package doesn't compile for the operating system you'd like to support. 

## 1. Fork the Repository
First, fork the repository to prepare your contributions. You can do that by following [this link](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/forks/new) or by clicking the "Forks" button on the repository homepage. After forking, clone the repository to your computer.

## 2. Create a new backend file
Create a new file for your IPC backend in `squiid-engine/src/ipc`. The default nng IPC is located in the `nng.rs` file in this directory. Create a new struct with a descriptive name matching your IPC protocol. 

## 3. Implement the IPCBackend trait
Now, you will need to implement the IPC trait. Here's what a basic example might look like:

```rs
pub struct MyNewIPCProtocol {}

impl IPCBackend for MyNewIPCProtocol {
    fn new() -> Self {
        todo!()
    }

    fn bind_and_listen(&self, address: &str) -> Result<..., ...> {
        todo!()
    }

    fn recv_data(&self) -> Result<..., ...> {
        todo!()
    }

    fn send_data(&self, response: ...) -> Result<..., ...> {
        todo!()
    }
}
```

This example won't compile because I shortened the function signatures. Once you type the `impl IPCBackend for ...`, your editor may be able to autocomplete the required functions and signatures. If not, check the `mod.rs` file for a reference.

## 4. Adding the file to the engine
Once you've completed the creation of your struct implementation, open `mod.rs` and add `pub mod filename;` at the top with `filename` being the name of the file you created. Above this mod declaration, add a feature flag above it.

## 5. Adding a new feature flag
Add a new feature flag under `[features]` in `Cargo.toml`. This will be similar to the name of your IPC backend, and it will depend on the "ipc" feature, as well as any extra dependencies. The default nng feature looks like this:

```toml
nng = ["ipc", "dep:nng"]
```

You can then set up the program to compile with default features disabled and your new IPC feature enabled. Make sure that the frontend you're using supports this IPC backend as well.