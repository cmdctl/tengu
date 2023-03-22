use anyhow::Result;
use terminal_ui::start_tui;

mod terminal_ui;

fn main() -> Result<()> {
    start_tui()
}
