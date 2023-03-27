use self::models::{InputMode, Tengu};
use self::repository::FsTenguRepository;
use self::ui::ui;
use anyhow::Result;
use crossterm::event::Event::Key;
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub mod models;
pub mod repository;
pub mod ui;

pub fn start_tui() -> Result<()> {
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

pub fn run_app(
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
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
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
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
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
                    KeyCode::Enter => {
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
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
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
