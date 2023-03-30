use std::{collections::HashMap, io, thread};

use lazy_static::lazy_static;
use unicode_width::UnicodeWidthStr;

use zmq::Socket;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use ratatui::{
    backend::Backend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::utils::{current_char_index, send_data};

#[derive(PartialEq)]
enum InputMode {
    None,
    Algebraic,
    RPN,
}

// RPN symbols and their corresponding commands
lazy_static! {
    static ref RPN_SYMBOL_MAP: HashMap<KeyCode, &'static str> = [
        (KeyCode::Char('+'), "add"),
        (KeyCode::Char('-'), "subtract"),
        (KeyCode::Char('*'), "multiply"),
        (KeyCode::Char('/'), "divide"),
        (KeyCode::Char('^'), "power"),
        (KeyCode::Char('_'), "invert"),
        (KeyCode::Char('\\'), "drop"),
    ]
    .iter()
    .copied()
    .collect();
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    history: Vec<String>,
    // Calculator info
    info: Vec<String>,
    // Stack for RPN mode
    stack: Vec<String>,
    // Most recent error message
    error: String,
    // current cursor offset
    left_cursor_offset: u16,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::None,
            history: Vec::new(),
            info: vec![
                format!(
                    "Squiid Calculator version {}",
                    option_env!("CARGO_PKG_VERSION").unwrap()
                ),
                "Copyright 2023 Connor Sample and Finian Wright".to_string(),
                "https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid".to_string(),
            ],
            stack: Vec::new(),
            error: String::new(),
            left_cursor_offset: 0,
        }
    }
}

fn update_stack_or_error(msg: String, app: &mut App) {
    if msg.starts_with("Error: ") {
        app.error = msg.clone();
    } else {
        app.stack = msg.split(",").map(|x| x.to_owned()).collect();
    }
}

// Handle algebraic expressions
fn algebraic_eval(mut app: &mut App, socket: &Socket) {
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
    }
    // Update stack
    update_stack_or_error(msg_as_str.clone(), &mut app);

    // Last item in stack is result of this expression
    let result = app.stack.last().unwrap();

    // Combine entry and result into line to print
    let mut history_entry = entered_expression;
    history_entry.push_str(" = ");
    history_entry.push_str(result);

    // Add to history
    app.history.push(history_entry);
}

// Handle typing in RPN mode
fn rpn_input(mut app: &mut App, socket: &Socket, c: char) {
    // Add character to input box
    let index = current_char_index(app.left_cursor_offset as usize, app.input.len());
    app.input.insert(index, c);

    // TODO: Add a way for the engine to send its command list
    let commands = [
        "add", "subtract", "multiply", "divide", "power", "sqrt", "mod", "sin", "cos", "tan",
        "sec", "csc", "cot", "asin", "acos", "atan", "acos", "atan", "log", "logb", "ln", "abs",
        "eq", "gt", "lt", "gte", "lte", "round", "invert", "drop", "swap", "dup", "rolldown",
        "rollup", "store", "clear", "quit",
    ];
    // Check if input box contains a command, if so, automatically execute it
    if commands.contains(&(app.input.as_str())) {
        // Send command
        let msg_as_str = send_data(socket, app.input.as_str());
        // Update stack display
        update_stack_or_error(msg_as_str, &mut app);
        // Clear input
        app.input.drain(..);
        // reset cursor offset
        app.left_cursor_offset = 0;
    }
}

// Handle RPN enter
fn rpn_enter(mut app: &mut App, socket: &Socket) {
    // Get command from input box and empty it
    let command: String = app.input.drain(..).collect();
    // reset cursor offset
    app.left_cursor_offset = 0;
    let msg_as_str;
    // Send command if there is one, otherwise duplicate last item in stack
    if command.len() > 0 {
        // Send to backend and get response
        msg_as_str = send_data(socket, command.as_str());
    } else {
        // Empty input, duplicate
        msg_as_str = send_data(socket, "dup");
    }
    // Update stack display
    update_stack_or_error(msg_as_str, &mut app);
}

// Handle RPN operators
fn rpn_operator(mut app: &mut App, socket: &Socket, key: crate::event::KeyEvent) {
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
        _ if RPN_SYMBOL_MAP.contains_key(&key.code) => RPN_SYMBOL_MAP.get(&key.code).unwrap(),
        _ => "there is no way for this to occur",
    };
    // Send operation
    let msg_as_str = send_data(socket, operation);
    // Update stack display
    update_stack_or_error(msg_as_str, &mut app);
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    socket: &Socket,
    backend_join_handle: &thread::JoinHandle<()>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if backend_join_handle.is_finished() {
            return Ok(());
        }

        // Handle keypresses
        if let Event::Key(key) = event::read()? {
            // Clear error message on keypress
            app.error = String::new();
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
                InputMode::Algebraic | InputMode::RPN if key.kind == KeyEventKind::Press => {
                    match key.code {
                        // Handle enter
                        KeyCode::Enter => {
                            if app.input_mode == InputMode::Algebraic {
                                algebraic_eval(&mut app, socket);
                            } else {
                                rpn_enter(&mut app, socket);
                            }
                        }
                        // Handle single character operators
                        _ if RPN_SYMBOL_MAP.contains_key(&key.code)
                            && app.input_mode == InputMode::RPN =>
                        {
                            rpn_operator(&mut app, socket, key);
                        }
                        // Handle typing characters
                        KeyCode::Char(c) => {
                            if app.input_mode == InputMode::Algebraic {
                                // Add character to input box
                                let index = current_char_index(
                                    app.left_cursor_offset as usize,
                                    app.input.len(),
                                );
                                app.input.insert(index, c);
                            } else if app.input_mode == InputMode::RPN {
                                rpn_input(&mut app, socket, c);
                            }
                        }
                        // Handle backspace
                        KeyCode::Backspace => {
                            // Remove character from input box
                            let index = current_char_index(
                                app.left_cursor_offset as usize,
                                app.input.len(),
                            );
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
                    }
                }
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
        _ if !app.error.is_empty() => (
            vec![Span::styled(
                app.error.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )],
            Style::default(),
        ),
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
        InputMode::Algebraic => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for options, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to evaluate"),
            ],
            Style::default(),
        ),
        InputMode::RPN => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for options, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to add to stack, or press or type an operation"),
            ],
            Style::default(),
        ),
    };

    // Set what to display in the upper box
    let mut display = match app.input_mode {
        InputMode::None => app.info.clone(),
        InputMode::Algebraic => app.history.clone(),
        InputMode::RPN => app.stack.clone(),
    };

    // Reverse display since we're rendering from the bottom
    display.reverse();

    let messages: Vec<ListItem> = display
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = Spans::from(Span::raw(format!(
                "{: >3}: {}",
                match app.input_mode {
                    InputMode::Algebraic | InputMode::RPN => i.to_string(),
                    InputMode::None => "".to_string(),
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
        InputMode::None => "Squiid",
    };
    let messages = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .start_corner(Corner::BottomLeft);

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
