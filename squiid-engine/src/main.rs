// pub mod lib;
use squiid_engine::start_server;
use std::env;
fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    let mut address = "tcp://*:33242";
    if args.len() > 1 {
        address = &args[1];
    }
    start_server(Some(address));
}
