use log::LevelFilter;
use rustyline::error::ReadlineError;

fn main() {
    env_logger::Builder::new()
        .filter_module("squiid_parser", LevelFilter::Debug)
        .init();
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(line) => {
                let rpn_expression = squiid_parser::parse(&line);
                for command_raw in rpn_expression.iter() {
                    let command = match command_raw.as_str() {
                        "+" => "add",
                        "-" => "subtract",
                        "*" => "multiply",
                        "/" => "divide",
                        "%" => "mod",
                        "^" => "power",
                        _ => command_raw,
                    };
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl+C. Exiting...");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl+D. Exiting...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
