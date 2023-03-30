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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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

struct StatefulTopPanel {
    state: ListState,
    items: Vec<String>,
}

impl StatefulTopPanel {
    fn with_items(items: Vec<String>) -> StatefulTopPanel {
        StatefulTopPanel {
            state: ListState::default(),
            items: items,
        }
    }

    fn next(&mut self, stack: &Vec<String>) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= stack.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self, stack: &Vec<String>) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    stack.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn deselect(&mut self) {
        self.state.select(None);
    }

    fn currently_selecting(&mut self) -> bool {
        self.state.selected().is_some()
    }

    fn selected_item(&mut self) -> String {
        let selected_index = self.state.selected();
        match selected_index {
            Some(index) => self.items[index].clone(),
            None => "".to_string(),
        }
    }
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
    // stack selection index
    top_panel_state: StatefulTopPanel,
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
            top_panel_state: StatefulTopPanel::with_items(vec![]),
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

    // Placeholder result of none in case there is nothing on the stack
    let mut result = "None";
    // Set result to last item in stack if there is one
    if app.stack.len() > 0 {
        result = app.stack.last().unwrap();
    }

    // Combine entry and result into line to print
    let mut history_entry = entered_expression;
    if app.error.is_empty() {
        history_entry.push_str(" = ");
        history_entry.push_str(result);
    } else {
        history_entry.push_str(" : ");
        history_entry.push_str(app.error.as_str());
    }

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
                            if app.top_panel_state.currently_selecting() {
                                // currently selecting, insert into text
                                let selected_item = app
                                    .top_panel_state
                                    .selected_item()
                                    .split_once(": ")
                                    .unwrap()
                                    .1
                                    .to_string();

                                let mut index = current_char_index(
                                    app.left_cursor_offset as usize,
                                    app.input.len(),
                                );
                                for char in selected_item.chars() {
                                    app.input.insert(index, char);
                                    index += 1;
                                }

                                app.top_panel_state.deselect();
                            } else if app.input_mode == InputMode::Algebraic {
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
                        KeyCode::PageUp => {
                            update_stack_or_error(send_data(socket, "rollup"), &mut app)
                        }
                        KeyCode::PageDown => {
                            update_stack_or_error(send_data(socket, "rolldown"), &mut app)
                        }
                        KeyCode::Tab => update_stack_or_error(send_data(socket, "swap"), &mut app),
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
                            // currently selecting; deselect
                            if app.top_panel_state.currently_selecting() {
                                app.top_panel_state.deselect();
                            } else {
                                // Return to normal mode
                                app.input_mode = InputMode::None;
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
                        // up keypress
                        KeyCode::Up => {
                            if app.input_mode == InputMode::RPN {
                                app.top_panel_state.next(&app.stack);
                            } else if app.input_mode == InputMode::Algebraic {
                                app.top_panel_state.next(&app.history);
                            }
                        }
                        // Down keypress
                        KeyCode::Down => {
                            if app.input_mode == InputMode::RPN {
                                app.top_panel_state.previous(&app.stack);
                            } else if app.input_mode == InputMode::Algebraic {
                                app.top_panel_state.previous(&app.history);
                            }
                        }
                        // Ignore all other keys
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        // Update stack if there is currently an error, since the last request will have gotten the error not the stack
        if !app.error.is_empty() {
            let msg = send_data(socket, "refresh");
            app.stack = msg.split(",").map(|x| x.to_owned()).collect();
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

    // Set help message to display
    let (msg, style) = match app.input_mode {
        // Display error if there is one
        _ if !app.error.is_empty() => (
            vec![Span::styled(
                app.error.clone(),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Red)
                    .bg(Color::White),
            )],
            Style::default(),
        ),
        _ if app.top_panel_state.currently_selecting() => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to insert the selected option, "),
            ],
            Style::default(),
        ),
        // Display help for options screen
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
        // Display help for algebraic mode
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
        // Display help for RPN mode
        InputMode::RPN => (
            vec![
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": options  "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": add to stack  "),
                Span::styled(
                    "Page Up/Down",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": roll stack  "),
                Span::styled("\\", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": drop  "),
                Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": swap"),
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
    app.top_panel_state.items.clear();

    let top_panel_content: Vec<ListItem> = display
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let displayed_string = format!(
                "{: >3}: {}",
                match app.input_mode {
                    InputMode::Algebraic | InputMode::RPN => i.to_string(),
                    InputMode::None => "".to_string(),
                },
                m
            );
            app.top_panel_state.items.push(displayed_string.clone());
            let content = Spans::from(Span::raw(displayed_string));
            ListItem::new(content)
        })
        .collect();

    // app.top_panel_state = StatefulTopPanel::with_items(top_panel_content);

    // Change title based on input mode
    let list_title = match app.input_mode {
        _ if app.top_panel_state.currently_selecting() => "Select",
        InputMode::Algebraic => "History",
        InputMode::RPN => "Stack",
        InputMode::None => "Squiid",
    };

    let top_panel = List::new(top_panel_content)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ")
        .start_corner(Corner::BottomLeft);

    if app.top_panel_state.currently_selecting() {
        f.render_stateful_widget(
            top_panel.style(Style::default().fg(Color::Blue)),
            chunks[0],
            &mut app.top_panel_state.state,
        );
    } else {
        f.render_stateful_widget(top_panel, chunks[0], &mut app.top_panel_state.state);
    }

    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[1]);

    let input_label = match app.input_mode {
        InputMode::Algebraic => "Algebraic",
        InputMode::RPN => "RPN",
        _ => "If this message appears, you have broken something",
    };

    if app.input_mode == InputMode::Algebraic || app.input_mode == InputMode::RPN {
        let input = Paragraph::new(app.input.as_ref())
            .style(match app.input_mode {
                _ if app.top_panel_state.currently_selecting() => Style::default(),
                InputMode::None => Style::default(),
                InputMode::Algebraic => Style::default().fg(Color::Yellow),
                InputMode::RPN => Style::default().fg(Color::Red),
            })
            .block(Block::default().borders(Borders::ALL).title(input_label));
        f.render_widget(input, chunks[2]);
    }
    match app.input_mode {
        InputMode::None =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Algebraic | InputMode::RPN if !app.top_panel_state.currently_selecting() => {
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

        _ => (),
    }
}
