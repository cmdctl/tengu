use std::io;

use anyhow::Result;
use crossterm::event::Event::Key;
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use repository::{FsTenguRepository, TenguRepository};
use serde::{Deserialize, Serialize};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

mod repository;

const APP_KEYS_DESC: &str = r#"
USAGE:
l:           List
a:           On list, It's Activate connection
d:           On list, It's Delete connection
e:           On list, It's Edit connection
s:           Search
i:           Insert new Connection
Tab:         Go to next field
Shift+Tab:   Go to previous filed
Esc:         Exit insert mode
"#;

enum InputMode {
    Normal,
    Name,
    Engine,
    Host,
    Port,
    Username,
    Password,
    Database,
    Submit,
    Search,
    List,
    Delete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    name: String,
    engine: String,
    host: String,
    port: String,
    username: String,
    password: String,
    database: String,
}

impl Connection {
    pub fn new(
        name: String,
        engine: String,
        host: String,
        port: String,
        username: String,
        password: String,
        database: String,
    ) -> Connection {
        Connection {
            name,
            engine,
            host,
            port,
            username,
            password,
            database,
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.engine, self.username, self.password, self.host, self.port, self.database
        )
    }
}

struct Tengu<R: TenguRepository> {
    repo: R,
    mode: InputMode,
    list_state: ListState,
    connections: Vec<Connection>,
    search_txt: String,
    search_list: Vec<Connection>,
    new_name: String,
    new_engine: String,
    new_host: String,
    new_port: String,
    new_username: String,
    new_password: String,
    new_database: String,
    edit_mode: bool,
    edit_index: Option<usize>,
    active_connection: Option<Connection>,
}

impl Tengu<FsTenguRepository> {
    pub fn new() -> Tengu<FsTenguRepository> {
        let repo = FsTenguRepository::new();
        let connections = repo.list();
        Tengu {
            repo,
            mode: InputMode::Normal,
            list_state: ListState::default(),
            connections,
            search_txt: String::new(),
            search_list: Vec::new(),
            new_name: String::new(),
            new_engine: String::new(),
            new_host: String::new(),
            new_port: String::new(),
            new_username: String::new(),
            new_password: String::new(),
            new_database: String::new(),
            edit_mode: false,
            edit_index: None,
            active_connection: None,
        }
    }
    pub fn change_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }
    pub fn clear_fields(&mut self) {
        self.new_name.clear();
        self.new_engine.clear();
        self.new_host.clear();
        self.new_port.clear();
        self.new_username.clear();
        self.new_password.clear();
        self.new_database.clear();
    }

    pub fn insert(&mut self) {
        let connection = Connection::new(
            self.new_name.clone(),
            self.new_engine.clone(),
            self.new_host.clone(),
            self.new_port.clone(),
            self.new_username.clone(),
            self.new_password.clone(),
            self.new_database.clone(),
        );
        self.repo.insert(&connection);
        self.connections.push(connection);
        self.clear_fields();
        self.change_mode(InputMode::Normal);
    }
    pub fn start_edit_mode(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let connection = &self.connections[index];
            self.new_name = connection.name.to_owned();
            self.new_engine = connection.engine.to_owned();
            self.new_host = connection.host.to_owned();
            self.new_port = connection.port.to_owned();
            self.new_username = connection.username.to_owned();
            self.new_password = connection.password.to_owned();
            self.new_database = connection.database.to_owned();
            self.edit_mode = true;
            self.edit_index = Some(index);
            self.change_mode(InputMode::Name);
        }
    }
    pub fn edit(&mut self) {
        let index = self.edit_index.unwrap();
        let connection = Connection::new(
            self.new_name.to_owned(),
            self.new_engine.to_owned(),
            self.new_host.to_owned(),
            self.new_port.to_owned(),
            self.new_username.to_owned(),
            self.new_password.to_owned(),
            self.new_database.to_owned(),
        );
        self.repo.update(&connection);
        self.connections[index] = connection;
        self.clear_fields();
        self.end_edit_mode();
        self.change_mode(InputMode::List);
    }
    pub fn end_edit_mode(&mut self) {
        if self.edit_mode {
            self.edit_mode = false;
            self.edit_index = None;
        }
    }
    pub fn check_delete(&mut self) {
        if self.list_state.selected().is_some() {
            self.change_mode(InputMode::Delete);
        }
    }
    pub fn delete(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let name = self.connections[index].name.to_owned();
            self.connections.remove(index);
            self.repo.delete(name);
            if index > 0 {
                self.list_state.select(Some(0));
            } else {
                self.list_state.select(None);
            }
            self.change_mode(InputMode::List);
        }
    }
    pub fn search(&mut self) {
        self.search_list = self
            .connections
            .clone()
            .into_iter()
            .filter(|item| item.name.starts_with(&self.search_txt.to_owned()))
            .collect();
    }
    pub fn move_up(&mut self) {
        let selected = match self.list_state.selected() {
            Some(v) => {
                if v == 0 {
                    Some(v)
                } else {
                    Some(v - 1)
                }
            }
            None => Some(0),
        };
        self.list_state.select(selected);
    }
    pub fn move_down(&mut self) {
        let selected = match self.list_state.selected() {
            Some(v) => {
                if v == self.connections.len() - 1 {
                    Some(v)
                } else {
                    Some(v + 1)
                }
            }
            None => Some(0),
        };
        self.list_state.select(selected);
    }
    pub fn activate_connection(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let connection = self.connections[index].clone();
            self.repo.activate_connection(&connection);
            self.active_connection = Some(connection);
        }
    }
}

fn start_tui() -> Result<()> {
    let mut state = Tengu::new();
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen // DisableMouseCapture
    )?;

    if let Err(e) = result {
        println!("{}", e.to_string());
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut Tengu<FsTenguRepository>,
) -> Result<(), std::io::Error> {
    loop {
        terminal.draw(|f| ui(f, state))?;

        if let Key(key) = event::read()? {
            match state.mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('s') => {
                        state.change_mode(InputMode::Search);
                    }
                    KeyCode::Char('l') => {
                        state.change_mode(InputMode::List);
                    }
                    KeyCode::Char('i') => {
                        state.change_mode(InputMode::Name);
                    }
                    _ => {}
                },

                InputMode::Name => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_name.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_name.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Engine);
                    }
                    _ => {}
                },
                InputMode::Engine => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_engine.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_engine.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Host);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Name);
                    }
                    _ => {}
                },

                InputMode::Host => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_host.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_host.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Port);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Engine);
                    }
                    _ => {}
                },
                InputMode::Port => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_port.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_port.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Username);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Host);
                    }
                    _ => {}
                },

                InputMode::Username => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_username.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_username.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Password);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Name);
                    }
                    _ => {}
                },

                InputMode::Password => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_password.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_password.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Database);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Username);
                    }
                    _ => {}
                },
                InputMode::Database => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_database.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_database.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Submit);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Password);
                    }
                    _ => {}
                },

                InputMode::Submit => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Password);
                    }
                    KeyCode::Enter => {
                        if state.edit_mode {
                            state.edit();
                        } else {
                            state.insert();
                        }
                    }
                    _ => {}
                },

                InputMode::Search => match key.code {
                    KeyCode::Esc => {
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.search_txt.push(c);
                        state.search();
                    }
                    KeyCode::Backspace => {
                        state.search_txt.pop();
                        state.search();
                    }
                    _ => {}
                },

                InputMode::List => match key.code {
                    KeyCode::Esc => {
                        state.list_state.select(None);
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char('k') => {
                        state.move_up();
                    }
                    KeyCode::Char('j') => {
                        state.move_down();
                    }
                    KeyCode::Char('a') => {
                        state.activate_connection();
                    }
                    KeyCode::Char('e') => {
                        state.start_edit_mode();
                    }
                    KeyCode::Char('d') => {
                        state.check_delete();
                    }
                    _ => {}
                },

                InputMode::Delete => match key.code {
                    KeyCode::Char('n') => {
                        state.change_mode(InputMode::List);
                    }
                    KeyCode::Char('y') => {
                        state.delete();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut tui::Frame<CrosstermBackend<std::io::Stdout>>, state: &mut Tengu<FsTenguRepository>) {
    let parent_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let new_section_block = Block::default()
        .title("New Connection")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(match state.mode {
            InputMode::Name
            | InputMode::Engine
            | InputMode::Host
            | InputMode::Port
            | InputMode::Username
            | InputMode::Password
            | InputMode::Database
            | InputMode::Submit => Style::default().fg(Color::LightGreen),
            _ => Style::default(),
        });
    f.render_widget(new_section_block, parent_chunk[0]);
    new_section(f, state, parent_chunk[0]);

    let list_section_block = Block::default()
        .title("List of Connections")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(match state.mode {
            InputMode::List | InputMode::Delete | InputMode::Search => {
                Style::default().fg(Color::LightGreen)
            }
            _ => Style::default(),
        });
    f.render_widget(list_section_block, parent_chunk[1]);
    list_section(f, state, parent_chunk[1]);

    delete_popup(f, state);
}

fn delete_popup(f: &mut Frame<CrosstermBackend<io::Stdout>>, state: &mut Tengu<FsTenguRepository>) {
    if let InputMode::Delete = state.mode {
        let block = Block::default()
            .title("DELETE")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let area = centered_rect(60, 25, f.size());
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);

        let chunk = Layout::default()
            .margin(2)
            .constraints([Constraint::Length(2), Constraint::Length(2)].as_ref())
            .split(area);

        let text = Paragraph::new("Are you sure?")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(text, chunk[0]);

        let keys_desc =
            Paragraph::new("Press (Y) for Yes and (N) for No").alignment(Alignment::Center);
        f.render_widget(keys_desc, chunk[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn list_section(
    f: &mut Frame<CrosstermBackend<io::Stdout>>,
    state: &mut Tengu<FsTenguRepository>,
    area: Rect,
) {
    let list_to_show = if state.search_list.is_empty() {
        state.connections.to_owned()
    } else {
        state.search_list.to_owned()
    };
    let active_connection = state.repo.get_active_connection();
    let items: Vec<ListItem> = list_to_show
        .into_iter()
        .map(|item| match state.mode {
            InputMode::List => ListItem::new(format!(
                "{} {}: {}",
                active_connection.clone().map_or("".to_string(), |c| {
                    if c.name == item.name {
                        " (active)".to_string()
                    } else {
                        "".to_string()
                    }
                }),
                item.name.to_owned(),
                item.engine,
            )),
            _ => ListItem::new(Span::from(item.name)),
        })
        .collect();

    let list_chunks = Layout::default()
        .margin(2)
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area);

    let search_input = Paragraph::new(state.search_txt.to_owned())
        .block(
            Block::default()
                .title("Search")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Search => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(search_input, list_chunks[0]);

    let list = List::new(items)
        .block(Block::default())
        .highlight_symbol("->")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, list_chunks[1], &mut state.list_state);
}

fn new_section(
    f: &mut tui::Frame<CrosstermBackend<std::io::Stdout>>,
    state: &mut Tengu<FsTenguRepository>,
    area: Rect,
) {
    let new_section_chunk = Layout::default()
        .margin(2)
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);
    let active_connection = state.repo.get_active_connection();
    let active_connection_paragraph = Paragraph::new(active_connection.map_or("".to_string(), |c| {
        format!("Active connection: {} {}", c.name, c.engine)
    })).style(Style::default().fg(Color::LightGreen));

    let desc = Paragraph::new(APP_KEYS_DESC).style(Style::default().fg(Color::LightMagenta));
    f.render_widget(desc, new_section_chunk[0]);
    f.render_widget(active_connection_paragraph, new_section_chunk[1]);

    let name_input = Paragraph::new(state.new_name.to_owned())
        .block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Name => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(name_input, new_section_chunk[2]);

    let engine_input = Paragraph::new(state.new_engine.to_owned())
        .block(
            Block::default()
                .title("Engine")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Engine => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(engine_input, new_section_chunk[3]);

    let host_input = Paragraph::new(state.new_host.to_owned())
        .block(
            Block::default()
                .title("Host")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Host => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(host_input, new_section_chunk[4]);

    let port_input = Paragraph::new(state.new_port.to_owned())
        .block(
            Block::default()
                .title("Port")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Port => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(port_input, new_section_chunk[5]);

    let username_input = Paragraph::new(state.new_username.to_owned())
        .block(
            Block::default()
                .title("Username")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Username => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(username_input, new_section_chunk[6]);

    let password_input = Paragraph::new(state.new_password.to_owned())
        .block(
            Block::default()
                .title("Password")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Password => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(password_input, new_section_chunk[7]);

    let database_input = Paragraph::new(state.new_database.to_owned())
        .block(
            Block::default()
                .title("Database")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Database => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(database_input, new_section_chunk[8]);

    let submit_btn = Paragraph::new("Submit")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Submit => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(submit_btn, new_section_chunk[9]);
}

fn main() -> Result<()> {
    start_tui()
}
