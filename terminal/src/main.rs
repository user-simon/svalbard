//! This is the terminal front-end for the Svalbard program. It has two main modes of operation: 
//! * TUI: the interface is drawn onto the terminal window with ASCII graphics and navigated with keyboard input.
//! * CLI: the program accepts, interprets, and executes command-line arguments.

mod cli;
mod tui;
mod shared;

use anyhow::Result;
use std::env;

fn main() -> Result<()> {
    if env::args().len() > 2 {
        cli::launch()
    } else {
        tui::launch()
    }
}
