use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use tui::{
    style::{Color, Style, Modifier},
    layout::{Constraint, Layout, Margin},
    widgets::{Block, Borders, BorderType, Clear}
};
use super::{
    state::{self, State, Status},
    Terminal,
    input::{self, Form, FormWidget},
    Frame,
    utility::{WrappedString, Center},
};

/// Displays a warning and returns whether the user confirmed.
pub fn confirm<S>(term: &mut Terminal, bg: Option<&mut dyn State>, msg: S) -> Result<bool>
where
    S: Into<String>
{
    let value = dialog(term, bg, DialogContent::Confirm(msg.into()))?
        .is_some();
    Ok(value)
}

/// Displays an info dialog until a key is pressed.
pub fn info<S>(term: &mut Terminal, bg: Option<&mut dyn State>, msg: S) -> Result<()>
where
    S: Into<String>
{
    notice(term, bg, NoticeLevel::Info, msg)
}

/// Displays a warning dialog until a key is pressed.
pub fn warning<S>(term: &mut Terminal, bg: Option<&mut dyn State>, msg: S) -> Result<()>
where
    S: Into<String>
{
    notice(term, bg, NoticeLevel::Warning, msg)
}

/// Displays an error dialog until a key is pressed.
pub fn error<S>(term: &mut Terminal, bg: Option<&mut dyn State>, msg: S) -> Result<()>
where
    S: Into<String>
{
    notice(term, bg, NoticeLevel::Error, msg)
}

/// Displays a fatal error dialog until a key is pressed.
pub fn fatal<S>(term: &mut Terminal, msg: S) -> Result<()>
where
    S: Into<String>
{
    notice(term, None, NoticeLevel::Fatal, msg)
}

/// Displays a dialog with an input form. Depending on how the user exits the dialog, the form is
/// returned for inspection.
pub fn form(term: &mut Terminal, bg: Option<&mut dyn State>, form: Form) -> Result<Option<Form>> {
    match dialog(term, bg, DialogContent::Form(form))? {
        Some(DialogContent::Form(form)) =>
            Ok(Some(form)),
        _ =>
            Ok(None)
    }
}

/// Displays a dialog with a message of a certain priority level specified by `NoticeLevel`.
fn notice<S>(term: &mut Terminal, bg: Option<&mut dyn State>, level: NoticeLevel, msg: S) -> Result<()>
where
    S: Into<String>
{
    dialog(term, bg, DialogContent::Notice(level, msg.into())).map(|_| ())
}

/// Displays a dialog with specified contents. Depending on how the user exits the dialog, the
/// content is returned for inspection.
fn dialog(term: &mut Terminal, bg: Option<&mut dyn State>, content: DialogContent) -> Result<Option<DialogContent>> {
    let state = state::exec(term, Dialog{ content, bg })?;
    Ok(state.map(|d| d.content))
}

/// Defines the different levels of notices a dialog can display.
enum NoticeLevel {
    Info,
    Warning,
    Error,
    Fatal,
}

/// Defines what may be contained within a dialog.
enum DialogContent {
    Confirm(String),
    Form(input::Form),
    Notice(NoticeLevel, String),
}

struct Dialog<'a> {
    /// Contains the content of the dialog.
    content: DialogContent,
    /// Drawn before the dialog, such that the dialog overlays the `bg` state. 
    bg: Option<&'a mut dyn State>, 
}

impl<'a> State for Dialog<'a> {
    fn update(&mut self, _: &mut Terminal, key: KeyCode, modifiers: KeyModifiers) -> Result<Status> {
        let status = match &mut self.content {
            DialogContent::Confirm(..) => {
                match key {
                    KeyCode::Char('y') |
                    KeyCode::Char('Y') => Status::Done,
                    KeyCode::Esc       |
                    KeyCode::Char('n') |
                    KeyCode::Char('N') => Status::Cancelled,
                    _                  => Status::Running,
                }
            },
            DialogContent::Form(form) => {
                match key {
                    KeyCode::Esc   => Status::Cancelled,
                    KeyCode::Enter => Status::Done,
                    _ => {
                        form.key_down(key, modifiers);
                        Status::Running
                    }
                }
            },
            DialogContent::Notice(..) => {
                Status::Done
            },
        };
        Ok(status)
    }
    
    fn draw(&mut self, frame: &mut Frame) {
        if let Some(bg) = &mut self.bg {
            bg.draw(frame);
        }

        // draw dialog container and return inner rectangle
        let inner = {
            let (title, style) = match &self.content {
                DialogContent::Notice(level, _) => {
                    let (title, color) = match level {
                        NoticeLevel::Info    => ("Info",        Color::Cyan),
                        NoticeLevel::Warning => ("Warning",     Color::Yellow),
                        NoticeLevel::Error   => ("Error",       Color::Red),
                        NoticeLevel::Fatal   => ("Fatal Error", Color::Red),
                    };
                    (title, Style::default().fg(color))
                }
                DialogContent::Form(form) => {
                    (form.title(), Style::default())
                }
                DialogContent::Confirm(_) => {
                    ("Confirm", Style::default().fg(Color::Yellow))
                }
            };
            
            let width = (frame.size().width as f32 * 0.6) as u16;
            let area = Layout::default()
                .horizontal_margin((frame.size().width - width) / 2)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25)
                ])
                .split(frame.size())[1];
            let block = Block::default()
                .borders(Borders::ALL)
                .style(style)
                .title(format!(" {} ", title.to_uppercase()))
                .border_type(BorderType::Thick);
            let inner = block.inner(area);
            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            inner.inner(&Margin{ horizontal: 3, vertical: 1 })
        };
        
        // draw content
        match &mut self.content {
            DialogContent::Confirm(msg) => {
                let hint = "Press (y) to confirm, (n) to cancel...";
                let msg_widget = WrappedString::new(&msg, inner.width)
                    .center();
                let hint_widget = WrappedString::new(hint, inner.width)
                    .style(Style::default().add_modifier(Modifier::ITALIC));
                let layout = Layout::default()
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(hint_widget.height()),
                    ])
                    .split(inner);
                frame.render_widget(msg_widget, layout[0]);
                frame.render_widget(hint_widget, layout[1]);
            }
            DialogContent::Notice(_, msg) => {
                let hint = "Press any key to close...";
                let msg_widget = WrappedString::new(&msg, inner.width)
                    .center();
                let hint_widget = WrappedString::new(hint, inner.width)
                    .style(Style::default().add_modifier(Modifier::ITALIC));
                let layout = Layout::default()
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(hint_widget.height()),
                    ])
                    .split(inner);
                frame.render_widget(msg_widget, layout[0]);
                frame.render_widget(hint_widget, layout[1]);
            }
            DialogContent::Form(form) => {
                let widget = FormWidget(form).center();
                frame.render_widget(widget, inner);
            }
        };
    }
}
