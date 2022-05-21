use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
pub use crossterm::event::{KeyCode, KeyModifiers};
use thiserror::Error;
use super::{Frame, Terminal};

/// Signal that the program should exit normally.
/// 
/// Defined as an Error to allow the use of `?` as a signal transmitter. 
#[derive(Debug, Error)]
#[error("")]
pub struct ExitSignal;

/// Communicates the status of an executing state to determine when and what to return from [exec].
pub enum Status {
    /// The state is finished and should be returned.
    Done,
    /// The state is finished but should be ignored.
    Cancelled,
    /// The state should continue running.
    Running,
}

/// Provides a common interface between all states used in [exec].
pub trait State {
    fn update(&mut self, term: &mut Terminal, key: KeyCode, modifiers: KeyModifiers) -> Result<Status>;
    fn draw(&mut self, frame: &mut Frame);
}

/// Main loop for each state. Called recursively for state transitions, thereby preserving the state
/// history on the stack and allowing the retrieval of state-data (such as forms) through the return
/// value.
/// 
/// This function is wrapped by ones representing individual states for brevity. E.g. to display an
/// informational dialog you should call the function [dialog::info](super::dialog::info) which
/// returns once the dialog has been exited.
/// 
/// # Returns
/// * `Some(state)` if the [state] exits with `Status::Done`.
/// * `None` if the [state] exits with `Status::Cancelled`.
pub fn exec<S>(term: &mut Terminal, mut state: S) -> Result<Option<S>>
where
    S: State
{
    loop {
        term.draw(|frame| state.draw(frame))?;
        
        if let Event::Key(KeyEvent { code, modifiers }) = event::read()? {
            match state.update(term, code, modifiers)? {
                Status::Done      => break Ok(Some(state)),
                Status::Cancelled => break Ok(None),
                Status::Running   => ()
            }
        }
    }
}
