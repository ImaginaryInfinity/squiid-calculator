/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, thread};
use unicode_width::UnicodeWidthStr;

use zmq::Socket;

#[derive(PartialEq)]
enum InputMode {
    None,
    Algebraic,
    RPN,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    // Stack for RPN mode
    stack: Vec<String>,
    // current cursor offset
    left_cursor_offset: u16,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::None,
            messages: Vec::new(),
            stack: Vec::new(),
            left_cursor_offset: 0,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // start evaluation server
    thread::spawn(|| {
        squiid_engine::start_server(Some("tcp://*:33242"));
    });

    // initiate zmq connection
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();
    assert!(socket.connect("tcp://localhost:33242").is_ok());

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app, &socket);

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

// Send data to backend
fn send_data(socket: &Socket, command: &str) -> String {
    let mut msg = zmq::Message::new();
    let _ = socket.send(command, 0);
    let _ = socket.recv(&mut msg, 0);
    msg.as_str().unwrap().to_string()
}

// get current character index based on cursor position and text length
fn current_char_index(left_cursor_offset: usize, input_len: usize) -> usize {
    let index: usize;
    if left_cursor_offset > input_len {
        index = 0;
    } else {
        index = input_len - left_cursor_offset;
    }

    index
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    socket: &Socket,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Handle keypresses
        if let Event::Key(key) = event::read()? {
            // Determine which mode the calculator is in
            match app.input_mode {
                // Handle keypresses for normal (non-editing) mode
                InputMode::None => match key.code {
                    KeyCode::Char('a') => {
                        app.input_mode = InputMode::Algebraic;
                    }
                    KeyCode::Char('r') => {
                        app.input_mode = InputMode::RPN;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                // Handle keypresses for algebraic input mode
                InputMode::Algebraic if key.kind == KeyEventKind::Press => match key.code {
                    // Handle enter
                    KeyCode::Enter => {
                        // Get string from input box and empty it
                        let entered_expression: String = app.input.drain(..).collect();
                        // reset cursor offset
                        app.left_cursor_offset = 0;
                        // Parse algebraic expression into postfix expression
                        let rpn_expression = squiid_parser::parse(entered_expression.trim());
                        // Create variable to store result from engine
                        let mut msg_as_str = String::new();

                        // Iterate through expression
                        for command_raw in rpn_expression.iter() {
                            // Convert operator symbols to engine commands
                            let command = match command_raw.as_str() {
                                "+" => "add",
                                "-" => "subtract",
                                "*" => "multiply",
                                "/" => "divide",
                                "^" => "power",
                                _ => command_raw,
                            };
                            // Send command to server
                            msg_as_str = send_data(socket, command);
                            // if msg_as_str == "quit" {
                            //     break 'input_loop;
                            // }
                        }
                        // Update stack
                        app.stack = msg_as_str.split(" ").map(|x| x.to_owned()).collect();

                        // Last item in stack is result of this expression
                        let result = app.stack.last().unwrap();

                        // Combine entry and result into line to print
                        let mut history_entry = entered_expression;
                        history_entry.push_str(" = ");
                        history_entry.push_str(result);

                        // Add to history
                        app.messages.push(history_entry);
                    }
                    // Handle typing characters
                    KeyCode::Char(c) => {
                        // Add character to input box
                        let index =
                            current_char_index(app.left_cursor_offset as usize, app.input.len());
                        app.input.insert(index, c);
                    }
                    // Handle backspace
                    KeyCode::Backspace => {
                        // Remove character from input box
                        let index =
                            current_char_index(app.left_cursor_offset as usize, app.input.len());
                        if index > 0 {
                            app.input.remove(index - 1);
                        }
                    }
                    // Handle escape
                    KeyCode::Esc => {
                        // Return to normal mode
                        app.input_mode = InputMode::None;
                    }
                    // left keypress
                    KeyCode::Left => {
                        // left arrow key, adjust left cursor offset
                        app.left_cursor_offset += 1;
                    }
                    // right keypress
                    KeyCode::Right => {
                        // right arrow key, adjust left cursor offset
                        if app.left_cursor_offset > 0 {
                            app.left_cursor_offset -= 1;
                        }
                    }
                    // Ignore all other keys
                    _ => {}
                },
                // Handle keypresses for RPN input mode
                InputMode::RPN if key.kind == KeyEventKind::Press => match key.code {
                    // Handle enter
                    KeyCode::Enter => {
                        // Get command from input box and empty it
                        let command: String = app.input.drain(..).collect();
                        // reset cursor offset
                        app.left_cursor_offset = 0;
                        let mut msg_as_str = String::new();
                        // Send command if there is one, otherwise duplicate last item in stack
                        if command.len() > 0 {
                            // Send to backend and get response
                            msg_as_str = send_data(socket, command.as_str());
                        } else {
                            // Empty input, duplicate
                            msg_as_str = send_data(socket, "dup");
                        }
                        // Update stack display
                        app.stack = msg_as_str.split(" ").map(|x| x.to_owned()).collect();
                    }
                    // Handle single character operators
                    KeyCode::Char('+')
                    | KeyCode::Char('-')
                    | KeyCode::Char('*')
                    | KeyCode::Char('/')
                    | KeyCode::Char('^')
                    | KeyCode::Char('_') => {
                        // Get operand from input box and empty it
                        let command: String = app.input.drain(..).collect();
                        // reset cursor offset
                        app.left_cursor_offset = 0;
                        // Send operand to backend if there is one
                        if command.len() > 0 {
                            _ = send_data(socket, command.as_str());
                        }
                        // Select operation
                        let operation = match key.code {
                            KeyCode::Char('+') => "add",
                            KeyCode::Char('-') => "subtract",
                            KeyCode::Char('*') => "multiply",
                            KeyCode::Char('/') => "divide",
                            KeyCode::Char('^') => "power",
                            KeyCode::Char('_') => "invert",
                            _ => "there is no way for this to occur",
                        };
                        // Send operation
                        let msg_as_str = send_data(socket, operation);
                        // Update stack display
                        app.stack = msg_as_str.split(" ").map(|x| x.to_owned()).collect();
                    }
                    // Handle typing characters
                    KeyCode::Char(c) => {
                        // Add character to input box
                        let index =
                            current_char_index(app.left_cursor_offset as usize, app.input.len());
                        app.input.insert(index, c);

                        // TODO: Add a way for the engine to send its command list
                        let commands = [
                            "add", "subtract", "multiply", "divide", "power", "sqrt", "mod", "sin",
                            "cos", "tan", "sec", "csc", "cot", "asin", "acos", "atan", "acos",
                            "atan", "log", "logb", "ln", "abs", "eq", "gt", "lt", "gte", "lte",
                            "round", "invert", "drop", "swap", "dup", "rolldown", "rollup",
                            "store", "clear", "quit",
                        ];
                        // Check if input box contains a command, if so, automatically execute it
                        if commands.contains(&(app.input.as_str())) {
                            // Send command
                            let msg_as_str = send_data(socket, app.input.as_str());
                            // Update stack display
                            app.stack = msg_as_str.split(" ").map(|x| x.to_owned()).collect();
                            // Clear input
                            app.input.drain(..);
                            // reset cursor offset
                            app.left_cursor_offset = 0;
                        }
                    }
                    // left keypress
                    KeyCode::Left => {
                        // left arrow key, adjust left cursor offset
                        app.left_cursor_offset += 1;
                    }
                    // right keypress
                    KeyCode::Right => {
                        // right arrow key, adjust left cursor offset
                        if app.left_cursor_offset > 0 {
                            app.left_cursor_offset -= 1;
                        }
                    }
                    // Handle backspace
                    KeyCode::Backspace => {
                        // Remove character from input box
                        let index =
                            current_char_index(app.left_cursor_offset as usize, app.input.len());
                        if index > 0 {
                            app.input.remove(index - 1);
                        }
                    }
                    // Handle escape
                    KeyCode::Esc => {
                        // Return to normal mode
                        app.input_mode = InputMode::None;
                    }
                    // Ignore all other keys
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::None => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for algebraic mode, "),
                Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for RPN mode."),
            ],
            Style::default(),
        ),
        InputMode::Algebraic | InputMode::RPN => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };

    let display = match app.input_mode {
        InputMode::None => &app.messages,
        InputMode::Algebraic => &app.messages,
        InputMode::RPN => &app.stack,
    };

    let messages: Vec<ListItem> = display
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = Spans::from(Span::raw(format!(
                "{}: {}",
                match app.input_mode {
                    InputMode::Algebraic => i,
                    InputMode::None => i,
                    InputMode::RPN => app.stack.len() - i,
                },
                m
            )));
            ListItem::new(content)
        })
        .collect();
    // Change title based on input mode
    let list_title = match app.input_mode {
        InputMode::Algebraic => "History",
        InputMode::RPN => "Stack",
        InputMode::None => "Messages",
    };
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title(list_title));
    f.render_widget(messages, chunks[0]);

    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[1]);

    if app.input_mode == InputMode::Algebraic || app.input_mode == InputMode::RPN {
        let input = Paragraph::new(app.input.as_ref())
            .style(match app.input_mode {
                InputMode::None => Style::default(),
                InputMode::Algebraic => Style::default().fg(Color::Yellow),
                InputMode::RPN => Style::default().fg(Color::Red),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[2]);
    }
    match app.input_mode {
        InputMode::None =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Algebraic | InputMode::RPN => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after rendering

            let mut cursor_position_x = chunks[2].x + app.input.width() as u16 + 1;
            if app.left_cursor_offset as usize > app.input.width() {
                app.left_cursor_offset = app.input.width() as u16;
            }

            cursor_position_x -= app.left_cursor_offset;
            f.set_cursor(
                // Put cursor past the end of the input text
                cursor_position_x,
                // Move one line down, from the border to the input line
                chunks[2].y + 1,
            )
        }
    }
}
