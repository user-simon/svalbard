mod dialog;
mod input;
mod state;
mod utility;
mod vault_view;

mod form; // TMP

use self::state::ExitSignal;
use crate::shared;
use anyhow::Result;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use tui::backend::CrosstermBackend;
use vault::Vault;

type Backend = tui::backend::CrosstermBackend<io::Stdout>;
type Terminal = tui::Terminal<Backend>;
type Frame<'a> = tui::Frame<'a, Backend>;

pub fn launch() -> Result<()> {
    // setup terminal environment
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen,)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    // start the business logic
    let result = ui(&mut term);

    // display error if one occurred
    if let Err(e) = result {
        if !e.is::<ExitSignal>() {
            dialog::fatal(&mut term, e.to_string()).expect(&format!(
                "An error occurred while trying to show you the error message: {e}"
            ));
        }
    }

    // restore terminal environment
    terminal::disable_raw_mode()?;
    crossterm::execute!(term.backend_mut(), LeaveAlternateScreen,)?;
    term.show_cursor()?;

    Ok(())
}

fn ui(term: &mut Terminal) -> Result<()> {
    let vault = Vault::load(&shared::vault_folder(), "😍".to_owned())?;
    vault_view::vault_view(term, vault, None)?;

    Ok(())
}
