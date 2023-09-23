use std::{error::Error, io, thread, time::Duration};

use clap::arg;
use nng::{Protocol, Socket};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
use app::{run_app, App};

mod config_utils;
mod utils;

use crossterm::{
    event::{self, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use squiid_engine::crash_reporter;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::command!()
        .args(&[arg!(-p --port [PORT] "an optional port number to use")])
        .get_matches();

    let specified_port = matches.get_one::<String>("port");

    // determine open TCP port
    let possible_port_num = match specified_port {
        Some(port) => Some(
            port.parse::<u16>()
                .expect("port argument must be an integer"),
        ),
        None => utils::get_available_port(20000..30000),
    };

    let port_num = match possible_port_num {
        Some(value) => value,
        None => return Err("Could not find open port in range 20000-30000".into()),
    };

    // start evaluation server
    let backend_join_handle = thread::spawn(move || {
        squiid_engine::start_server(Some(&format!("tcp://127.0.0.1:{}", port_num)));
    });

    // Wait for server to start
    thread::sleep(Duration::from_millis(10));

    // initiate nng connection
    let socket = Socket::new(Protocol::Req0).unwrap();
    assert!(socket
        .dial(&format!("tcp://127.0.0.1:{}", port_num))
        .is_ok());

    std::panic::set_hook(Box::new(|panic| {
        reset_terminal().unwrap();
        crash_reporter::crash_report(panic);
        std::process::exit(1);
    }));

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    print!("{}[2J", 27 as char);

    // create app and run it
    let app = App::new(&socket);
    let res = run_app(&mut terminal, app, &socket, &backend_join_handle);

    reset_terminal()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

/// Reset the terminal to the default state
fn reset_terminal() -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}
