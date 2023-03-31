use std::io;

use super::models::{Engine, InputMode, Tengu};
use super::repository::{FsTenguRepository, TenguRepository};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

const APP_KEYS_DESC: &str = r#"
USAGE:
l:           List
Enter:       On list, It's Activate connection
d:           On list, It's Delete connection
e:           On list, It's Edit connection
s:           Search
i:           Insert new Connection
Tab:         Go to next field
Shift+Tab:   Go to previous filed
Esc:         Exit insert mode
"#;

pub fn ui(
    f: &mut tui::Frame<CrosstermBackend<std::io::Stdout>>,
    state: &mut Tengu<FsTenguRepository>,
) {
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
    f.render_widget(new_section_block, parent_chunk[1]);
    new_section(f, state, parent_chunk[1]);

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
    f.render_widget(list_section_block, parent_chunk[0]);
    list_section(f, state, parent_chunk[0]);

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
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(6),
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
    let active_connection_paragraph =
        Paragraph::new(active_connection.map_or("".to_string(), |c| {
            format!("Active connection: {} {}", c.name, c.engine)
        }))
        .style(Style::default().fg(Color::LightGreen));

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

    let items = vec![
        ListItem::new(Engine::SqlServer.to_string()),
        ListItem::new(Engine::Postgres.to_string()),
        ListItem::new(Engine::Mysql.to_string()),
    ];
    let engine_list = List::new(items)
        .block(
            Block::default()
                .title("Engine")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_symbol("->")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .style(match state.mode {
            InputMode::Engine => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_stateful_widget(
        engine_list,
        new_section_chunk[3],
        &mut state.engines_list_state,
    );

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
