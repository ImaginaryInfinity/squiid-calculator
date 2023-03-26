use std::thread;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use zmq::Socket;

fn send_data(socket: &Socket, command: &str) -> String {
    let mut msg = zmq::Message::new();
    let _ = socket.send(command, 0);
    println!("{}", "waiting");
    let _ = socket.recv(&mut msg, 0);
    let msg_as_str = msg.as_str().unwrap();
    println!("{:?}", msg_as_str);
    msg.as_str().unwrap().to_string()
}

fn main() {
    // start evaluation server
    thread::spawn(|| {
        squiid_engine::start_server(Some("tcp://*:33242"));
    });

    // initiate zmq connection
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();

    assert!(socket.connect("tcp://localhost:33242").is_ok());

    // readline implementation
    let mut rl = DefaultEditor::new().unwrap();

    'input_loop: loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let rpn_expression = squiid_parser::parse(line.trim());

                for command_raw in rpn_expression.iter() {
                    let command = match command_raw.as_str() {
                        "+" => "add",
                        "-" => "subtract",
                        "*" => "multiply",
                        "/" => "divide",
                        "^" => "power",
                        _ => command_raw,
                    };
                    // println!("{}", command);

                    let msg_as_str = send_data(&socket, command);
                    if msg_as_str == "quit" {
                        break 'input_loop;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Press Ctrl-D or type quit to exit");
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl+D, Exiting...");
                send_data(&socket, "quit");
                break 'input_loop;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                send_data(&socket, "quit");
                break 'input_loop;
            }
        }
    }
}
