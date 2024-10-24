use std::{collections::HashMap, io};

use lazy_static::lazy_static;
use squiid_engine::{
    extract_data,
    protocol::server_response::{ResponsePayload, ResponseType, ServerResponseMessage},
};
use unicode_width::UnicodeWidthStr;

use nng::Socket;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::{
    config_utils,
    utils::{current_char_index, input_buffer_is_sci_notate, send_input_data},
};

/// The input mode state of the application
#[derive(PartialEq)]
enum InputMode {
    /// No input mode (select, info view, etc.)
    None,
    /// Algebraic input mode
    Algebraic,
    /// RPN input mode
    Rpn,
}

lazy_static! {
    /// RPN symbols and their corresponding commands
    static ref RPN_SYMBOL_MAP: HashMap<KeyCode, &'static str> = [
        (KeyCode::Char('+'), "add"),
        (KeyCode::Char('-'), "subtract"),
        (KeyCode::Char('*'), "multiply"),
        (KeyCode::Char('/'), "divide"),
        (KeyCode::Char('%'), "mod"),
        (KeyCode::Char('^'), "power"),
        (KeyCode::Char('<'), "lt"),
        (KeyCode::Char('>'), "gt"),
        (KeyCode::Char('_'), "chs"),
    ]
    .iter()
    .copied()
    .collect();
}

/// State of the selection view
struct StatefulTopPanel {
    /// State of selection
    state: ListState,
    /// The list of items that can be selected
    items: Vec<String>,
}

impl StatefulTopPanel {
    /// Initiate a new selection state from a Vec
    fn with_items(items: Vec<String>) -> StatefulTopPanel {
        StatefulTopPanel {
            state: ListState::default(),
            items,
        }
    }

    /// Move the selection to the next item
    fn next(&mut self, stack: &[String]) {
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

    /// Move the selection to the previous item
    fn previous(&mut self, stack: &[String]) {
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

    /// Deselect selected item
    fn deselect(&mut self) {
        self.state.select(None);
    }

    /// Check if an item is currently selected
    fn currently_selecting(&mut self) -> bool {
        self.state.selected().is_some()
    }

    /// Get the current selected item
    fn selected_item(&mut self) -> String {
        let selected_index = self.state.selected();
        match selected_index {
            Some(index) => self.items[index].clone(),
            None => "".to_string(),
        }
    }
}

/// App holds the state of the application
pub struct App<'a> {
    /// Current value of the input box
    input: String,
    /// Socket used to communicate with the backend
    pub socket: &'a Socket,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    history: Vec<String>,
    /// Calculator info
    info: Vec<String>,
    /// Stack for RPN mode
    stack: Vec<String>,
    /// Most recent error message
    error: String,
    /// Current cursor offset
    left_cursor_offset: u16,
    /// Stack selection state
    top_panel_state: StatefulTopPanel,
    quit_app: bool,
}

impl<'a> App<'a> {
    pub fn new(socket: &'a Socket) -> App<'a> {
        App {
            input: String::new(),
            socket,
            input_mode: InputMode::None,
            history: Vec::new(),
            info: vec![
                "     ;dOKNNNKkl.           .c.   ".to_string(),
                "   :NMXd:;,;lkWWk.        .c.    ".to_string(),
                "  .WM0        .OXl        :,     ".to_string(),
                "   NMX.                  ;:  ".to_string()
                    + &format!("Squiid Calculator version {}", env!("CARGO_PKG_VERSION")),
                "   .kWW0dc:,'.          ,c       ".to_string(),
                "      ;lxO0XWMXx.      .c.       ".to_string(),
                "             '0MMl     c'        ".to_string(),
                "   .           NMW    :;     ".to_string() + "        Copyright 2023",
                "  OMN:        ;WM0   ,:      ".to_string() + "Connor Sample and Finian Wright",
                "   cXMW0xoodkNMNo   'c.          ".to_string(),
                "     .:oxkOkdl'    .c.           ".to_string(),
                "                   :,            ".to_string(),
                "".to_string(),
                env!("CARGO_PKG_REPOSITORY").to_string(),
            ],
            stack: Vec::new(),
            error: String::new(),
            left_cursor_offset: 0,
            top_panel_state: StatefulTopPanel::with_items(vec![]),
            quit_app: false,
        }
    }
}

impl<'a> App<'a> {
    /// Get keybind from config file as string
    pub fn keybind_from_config(&mut self, keybind_name: &str) -> String {
        config_utils::get_key(self, "keybinds", keybind_name)
            .as_str()
            .unwrap()
            .to_string()
    }

    /// Get keycode from config
    pub fn keycode_from_config(&mut self, keybind_name: &str) -> KeyCode {
        let keybind = self.keybind_from_config(keybind_name);
        match keybind.as_str() {
            "backspace" => KeyCode::Backspace,
            "enter" => KeyCode::Enter,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "page_up" => KeyCode::PageUp,
            "page_down" => KeyCode::PageDown,
            "tab" => KeyCode::Tab,
            "backtab" => KeyCode::BackTab,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "escape" => KeyCode::Esc,
            _ if keybind.len() == 1 => KeyCode::Char(keybind.chars().next().unwrap()),
            _ => KeyCode::Null,
        }
    }
}

/// Update the stack if msg is not an error. If it is an error, display that error
pub fn update_stack_or_error(msg: ServerResponseMessage, app: &mut App) {
    // TODO: make a seperate display for commands
    match msg.response_type {
        ResponseType::Stack => {
            app.stack = extract_data!(msg.payload, ResponsePayload::Stack)
                .iter()
                .map(|item| item.to_string())
                .collect();
        }
        ResponseType::Error => {
            let error_message = extract_data!(msg.payload, ResponsePayload::Error);
            app.error = format!("Error: {}", error_message);
        }
        ResponseType::Commands => todo!(),
        ResponseType::QuitSig => app.quit_app = true,
        // configuration return is handeled elsewhere
        ResponseType::Configuration | ResponseType::PrevAnswer => (),
    }
}

/// Handle algebraic expressions
fn algebraic_eval(app: &mut App, socket: &Socket) {
    // Get string from input box and empty it
    let entered_expression: String = app.input.drain(..).collect();

    // Clear stack
    _ = send_input_data(socket, "clear");

    // Special frontend commands
    if entered_expression.as_str() == "clear" {
        app.history = Vec::new();
        return;
    };

    // reset cursor offset
    app.left_cursor_offset = 0;
    // Parse algebraic expression into postfix expression
    let rpn_expression = match squiid_parser::parse(entered_expression.trim()) {
        Ok(expr) => expr,
        Err(e) => {
            app.error = format!("Error: {}", e);
            return;
        }
    };

    // Commands that cannot be used in algebraic mode
    let non_algebraic_commands = [
        "invert", "drop", "swap", "dup", "rolldown", "rollup", "clear", "undo",
    ];
    // Iterate through the commands present in the expression
    for command_raw in rpn_expression.iter() {
        // Check if it is forbidden
        if non_algebraic_commands.contains(command_raw) {
            // Give error and stop trying to evaluate if the command is forbidden
            app.error = format!("Error: {} is invalid in algebraic mode", command_raw);
            return;
        }
    }

    // Iterate through expression
    for command_raw in rpn_expression.iter() {
        // Convert operator symbols to engine commands
        let command = match *command_raw {
            "+" => "add",
            "-" => "subtract",
            "*" => "multiply",
            "/" => "divide",
            "%" => "mod",
            "^" => "power",
            "=" => "invstore",
            "==" => "eq",
            ">" => "gt",
            "<" => "lt",
            ">=" => "geq",
            "<=" => "leq",
            _ => command_raw,
        };
        // Send command to server
        let msg = send_input_data(socket, command);
        // Update stack
        update_stack_or_error(msg, app);
    }

    // Empty placeholder result in case there is nothing on the stack
    let mut result = "";
    // Set result to last item in stack if there is one
    if !app.stack.is_empty() {
        result = app.stack.last().unwrap();
    }

    // Combine entry and result into line to print
    let mut history_entry = entered_expression;
    if app.error.is_empty() && !result.is_empty() {
        history_entry.push_str(" = ");
        history_entry.push_str(result);
    } else if !app.error.is_empty() {
        history_entry.push_str(" : ");
        history_entry.push_str(app.error.as_str());
    } else {
        history_entry.push_str(" : Done");
    }

    // Add to history
    app.history.push(history_entry);
}

/// Handle typing in RPN mode
fn rpn_input(app: &mut App, socket: &Socket, c: char) {
    // Add character to input box
    let index = current_char_index(app.left_cursor_offset as usize, app.input.len());
    app.input.insert(index, c);

    // query engine for available commands
    let binding = send_input_data(socket, "commands");
    let commands = extract_data!(binding.payload, ResponsePayload::Commands);

    // Check if input box contains a command, if so, automatically execute it
    if commands.contains(&app.input) {
        // Send command
        let msg = send_input_data(socket, app.input.as_str());
        // Update stack display
        update_stack_or_error(msg, app);
        // Clear input
        app.input.drain(..);
        // reset cursor offset
        app.left_cursor_offset = 0;
    }
}

/// Handle RPN enter
fn rpn_enter(app: &mut App, socket: &Socket) {
    // Get command from input box and empty it
    let command: String = app.input.drain(..).collect();
    // reset cursor offset
    app.left_cursor_offset = 0;
    // Send command if there is one, otherwise duplicate last item in stack
    let msg = if !command.is_empty() {
        // Send to backend and get response
        send_input_data(socket, command.as_str())
    } else {
        // Empty input, duplicate
        send_input_data(socket, "dup")
    };
    // Update stack display
    update_stack_or_error(msg, app);
}

/// Handle RPN operators
fn rpn_operator(app: &mut App, socket: &Socket, key: crate::event::KeyEvent) {
    // Get operand from input box and empty it
    let command: String = app.input.drain(..).collect();
    // reset cursor offset
    app.left_cursor_offset = 0;
    // Send operand to backend if there is one
    if !command.is_empty() {
        _ = send_input_data(socket, command.as_str());
    }

    // Select operation
    let operation = match key.code {
        _ if RPN_SYMBOL_MAP.contains_key(&key.code) => RPN_SYMBOL_MAP.get(&key.code).unwrap(),
        _ => "there is no way for this to occur",
    };
    // Send operation
    let msg = send_input_data(socket, operation);
    // Update stack display
    update_stack_or_error(msg, app);
}

/// Create the main application and run it
pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    socket: &Socket,
) -> io::Result<()> {
    // set default start mode
    let binding = config_utils::get_key(&mut app, "system", "start_mode");
    let start_mode = binding.as_str().unwrap();

    app.input_mode = match start_mode {
        "algebraic" => InputMode::Algebraic,
        "rpn" => InputMode::Rpn,
        _ => InputMode::None,
    };

    loop {
        if app.quit_app {
            return Ok(());
        }

        terminal.draw(|f| ui(f, &mut app))?;

        // Handle keypresses
        if let Event::Key(key) = event::read()? {
            // Clear error message on keypress
            app.error = String::new();
            // Determine which mode the calculator is in
            match app.input_mode {
                // Handle keypresses for normal (non-editing) mode
                InputMode::None => match key.code {
                    _ if key.code == app.keycode_from_config("mode_algebraic") => {
                        app.input_mode = InputMode::Algebraic;
                    }
                    _ if key.code == app.keycode_from_config("mode_rpn") => {
                        app.input_mode = InputMode::Rpn;
                    }
                    _ if key.code == app.keycode_from_config("quit") => {
                        return Ok(());
                    }
                    _ => {}
                },
                // Handle keypresses for algebraic input mode
                InputMode::Algebraic | InputMode::Rpn if key.kind == KeyEventKind::Press => {
                    match key.code {
                        // Handle enter
                        _ if key.code == app.keycode_from_config("enter") => {
                            send_input_data(socket, "update_previous_answer");

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
                            && app.input_mode == InputMode::Rpn
                            && !input_buffer_is_sci_notate(&app.input) =>
                        {
                            rpn_operator(&mut app, socket, key);
                        }

                        _ if key.code == app.keycode_from_config("rpn_drop")
                            && app.input_mode == InputMode::Rpn =>
                        {
                            update_stack_or_error(send_input_data(socket, "drop"), &mut app)
                        }

                        _ if key.code == app.keycode_from_config("rpn_roll_up")
                            && app.input_mode == InputMode::Rpn =>
                        {
                            update_stack_or_error(send_input_data(socket, "rollup"), &mut app)
                        }
                        _ if key.code == app.keycode_from_config("rpn_roll_down")
                            && app.input_mode == InputMode::Rpn =>
                        {
                            update_stack_or_error(send_input_data(socket, "rolldown"), &mut app)
                        }
                        _ if key.code == app.keycode_from_config("rpn_swap")
                            && app.input_mode == InputMode::Rpn =>
                        {
                            update_stack_or_error(send_input_data(socket, "swap"), &mut app)
                        }
                        _ if key.code == app.keycode_from_config("rpn_undo")
                            && app.input_mode == InputMode::Rpn =>
                        {
                            update_stack_or_error(send_input_data(socket, "undo"), &mut app)
                        }
                        _ if key.code == app.keycode_from_config("rpn_redo")
                            && app.input_mode == InputMode::Rpn =>
                        {
                            update_stack_or_error(send_input_data(socket, "redo"), &mut app)
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
                            } else if app.input_mode == InputMode::Rpn {
                                rpn_input(&mut app, socket, c);
                            }
                        }
                        // Handle backspace
                        KeyCode::Backspace => {
                            // Get current cursor position
                            let index = current_char_index(
                                app.left_cursor_offset as usize,
                                app.input.len(),
                            );
                            // Make sure a character exists to delete
                            if index > 0 {
                                // Remove character
                                app.input.remove(index - 1);
                            }
                        }
                        // Handle delete
                        KeyCode::Delete => {
                            // Get current cursor position
                            let index = current_char_index(
                                app.left_cursor_offset as usize,
                                app.input.len(),
                            );
                            // Make sure a character exists to delete
                            if app.input.len() > index {
                                // Remove character
                                app.input.remove(index);
                                // Resposition cursor
                                app.left_cursor_offset -= 1;
                            }
                        }
                        // Handle escape
                        _ if key.code == app.keycode_from_config("exit") => {
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
                        // Home key
                        KeyCode::Home => {
                            // Move cursor to beginning of line
                            app.left_cursor_offset = app.input.len() as u16;
                        }
                        // End key
                        KeyCode::End => {
                            // Move cursor to end of line
                            app.left_cursor_offset = 0;
                        }
                        // up keypress
                        KeyCode::Up => {
                            if app.input_mode == InputMode::Rpn && !app.stack.is_empty() {
                                app.top_panel_state.next(&app.stack);
                            } else if app.input_mode == InputMode::Algebraic
                                && !app.history.is_empty()
                            {
                                app.top_panel_state.next(&app.history);
                            }
                        }
                        // Down keypress
                        KeyCode::Down => {
                            if app.input_mode == InputMode::Rpn && !app.stack.is_empty() {
                                app.top_panel_state.previous(&app.stack);
                            } else if app.input_mode == InputMode::Algebraic
                                && !app.history.is_empty()
                            {
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
            let msg = send_input_data(socket, "refresh");
            app.stack = extract_data!(msg.payload, ResponsePayload::Stack)
                .iter()
                .map(|item| item.to_string())
                .collect();
        }
    }
}

/// Create the UI of the app
fn ui(f: &mut Frame, app: &mut App) {
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
        .split(f.area());

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
                Span::styled(
                    app.keybind_from_config("exit").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to exit, "),
                Span::styled(
                    app.keybind_from_config("enter").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to insert the selected option, "),
            ],
            Style::default(),
        ),
        // Display help for options screen
        InputMode::None => (
            vec![
                Span::raw("Press "),
                Span::styled(
                    app.keybind_from_config("quit").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to exit, "),
                Span::styled(
                    app.keybind_from_config("mode_algebraic").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" for algebraic mode, "),
                Span::styled(
                    app.keybind_from_config("mode_rpn").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" for RPN mode."),
            ],
            Style::default(),
        ),
        // Display help for algebraic mode
        InputMode::Algebraic => (
            vec![
                Span::raw("Press "),
                Span::styled(
                    app.keybind_from_config("exit").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" for options, "),
                Span::styled(
                    app.keybind_from_config("enter").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to evaluate"),
            ],
            Style::default(),
        ),
        // Display help for RPN mode
        InputMode::Rpn => (
            vec![
                Span::styled(
                    app.keybind_from_config("exit").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": options  "),
                Span::styled(
                    app.keybind_from_config("enter").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": enter in stack  "),
                Span::styled(
                    format!(
                        "{}/{}",
                        app.keybind_from_config("rpn_roll_up").to_owned(),
                        app.keybind_from_config("rpn_roll_down").to_owned()
                    ),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": roll stack  "),
                Span::styled(
                    app.keybind_from_config("rpn_drop").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": drop  "),
                Span::styled(
                    app.keybind_from_config("rpn_swap").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": swap  "),
                Span::styled(
                    app.keybind_from_config("rpn_undo").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": undo  "),
                Span::styled(
                    app.keybind_from_config("rpn_redo").to_owned(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(": redo"),
            ],
            Style::default(),
        ),
    };

    // Set what to display in the upper box
    let mut display = match app.input_mode {
        InputMode::None => app.info.clone(),
        InputMode::Algebraic => app.history.clone(),
        InputMode::Rpn => app.stack.clone(),
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
                    InputMode::Algebraic | InputMode::Rpn => i.to_string(),
                    InputMode::None => "".to_string(),
                },
                m
            );
            app.top_panel_state.items.push(displayed_string.clone());
            let content = Line::from(Span::raw(displayed_string));
            ListItem::new(content)
        })
        .collect();

    // app.top_panel_state = StatefulTopPanel::with_items(top_panel_content);

    // Change title based on input mode
    let list_title = match app.input_mode {
        _ if app.top_panel_state.currently_selecting() => "Select",
        InputMode::Algebraic => "History",
        InputMode::Rpn => "Stack",
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
        .direction(ListDirection::BottomToTop);

    if app.top_panel_state.currently_selecting() {
        f.render_stateful_widget(
            top_panel.style(Style::default().fg(Color::Blue)),
            chunks[0],
            &mut app.top_panel_state.state,
        );
    } else {
        f.render_stateful_widget(top_panel, chunks[0], &mut app.top_panel_state.state);
    }

    let mut text = Text::from(Line::from(msg));
    text = text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[1]);

    let input_label = match app.input_mode {
        InputMode::Algebraic => "Algebraic",
        InputMode::Rpn => "RPN",
        _ => "If this message appears, you have broken something",
    };

    if app.input_mode == InputMode::Algebraic || app.input_mode == InputMode::Rpn {
        let input_width = chunks[2].width as usize - 3; // Account for border characters

        // Determine the starting position of the text to display
        let mut start_pos = 0;
        if app.input.len() > input_width {
            // cursor_pos keeps track of the cursor position within the entire line of text
            // cursor_position_x keeps track of the x position of the rendered cursor
            let cursor_pos = app
                .input
                .len()
                .saturating_sub(app.left_cursor_offset as usize);
            if cursor_pos > input_width {
                start_pos = cursor_pos - input_width + 1;
            }
        }

        // Truncate and scroll the input text as needed
        let truncated_input = &app.input[start_pos.saturating_sub(1)..];

        // Calculate the cursor position based on the truncated input
        if app.left_cursor_offset as usize > app.input.len() {
            app.left_cursor_offset = app.input.len() as u16;
        }

        // calculate the rendered cursor's x position
        let cursor_position_x = chunks[2].x
            + (truncated_input.width() as u16).saturating_sub(app.left_cursor_offset)
            + 1;

        // THIS IS WHERE THE INPUT IS BEING ADDED TO THE PARAGRAPH DISPLAY
        let input = Paragraph::new(truncated_input)
            .style(match app.input_mode {
                _ if app.top_panel_state.currently_selecting() => Style::default(),
                InputMode::None => Style::default(),
                InputMode::Algebraic => Style::default().fg(Color::Yellow),
                InputMode::Rpn => Style::default().fg(Color::Red),
            })
            .block(Block::default().borders(Borders::ALL).title(input_label));
        f.render_widget(input, chunks[2]);

        // Set the cursor position
        f.set_cursor_position((cursor_position_x, chunks[2].y + 1));
    }
}
