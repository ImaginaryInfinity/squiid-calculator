use std::{error::Error, io};

use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
use app::{run_app, App};

mod config_handler;
mod utils;

use crossterm::{
    event::{self, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use squiid_engine::crash_reporter;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) || args.contains(&"-V".to_string()) {
        println!("squiid {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    config_handler::init_config();

    std::panic::set_hook(Box::new(|panic| {
        reset_terminal().unwrap();
        crash_reporter::crash_report(panic, Some(config_handler::determine_config_path()));
        std::process::exit(1);
    }));

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // clear terminal
    // this is for terminals that aren't smart enough to do it themselves
    // from whatever ratatui does
    print!("{}[2J", 27 as char);

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

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
