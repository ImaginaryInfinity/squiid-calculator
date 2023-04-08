use std::{error::Error, io, thread};

use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
use app::{run_app, App};

mod config_handler;
mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), Box<dyn Error>> {
    // determine open TCP port
    let possible_port_num = utils::get_available_port(20000..30000);

    let port_num = match possible_port_num {
        Some(value) => value,
        None => return Err("Could not find open port in range 20000-30000".into()),
    };

    // start evaluation server
    let backend_join_handle = thread::spawn(move || {
        squiid_engine::start_server(Some(&format!("tcp://127.0.0.1:{}", port_num)));
    });

    // initiate zmq connection
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();
    assert!(socket
        .connect(&format!("tcp://127.0.0.1:{}", port_num))
        .is_ok());

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app, &socket, &backend_join_handle);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
