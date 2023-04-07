use std::{error::Error, io, thread};

use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
use app::{run_app, App};

mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // start evaluation server
    let backend_join_handle = thread::spawn(|| {
        squiid_engine::start_server(Some("tcp://*:33242"));
    });

    // initiate zmq connection
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();
    assert!(socket.connect("tcp://localhost:33242").is_ok());

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
