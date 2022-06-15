use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher}, cell::{RefCell, Ref}, borrow::BorrowMut,
};

use super::{
    dialog,
    input::{self, Form, Input},
    state::{self, State, Status},
    Frame, Terminal,
};
use anyhow::Result;
use crossterm::event::{KeyModifiers, KeyCode};
use indoc::indoc;
use tui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};
use vault::{
    seed::{Characters, Seed},
    Vault,
};

pub fn vault_view(term: &mut Terminal, vault: Vault, key: Option<String>) -> Result<()> {
    let filter = input::StringInput::default();
    let (displayed, default_row) = filter_seeds(vault.seeds(), "");
    let mut table_state = TableState::default();
    table_state.select(default_row);
    let vault_hash = hash_vault(&vault);

    VaultView {
        vault,
        key,
        filter,
        displayed,
        table_state: RefCell::new(table_state),
        prev_vault_hash: vault_hash,
    }.exec(term)?;
    
    Ok(())
}

struct VaultView {
    /// The vault backend.
    vault: Vault,
    /// The key to be used generating passwords. If [None], is prompted when a password is
    /// generated.
    key: Option<String>,
    /// Text input containing a string to filter seeds by.
    filter: input::StringInput,
    /// Ordered indices of rows to display according to filter.
    displayed: Vec<usize>,
    /// Maintains index of the selected row. Uses [RefCell] for interior mutability for use in [State::draw].
    table_state: RefCell<TableState>,
    /// Used to check if the internal state has changed during runtime.
    prev_vault_hash: u64,
}

impl VaultView {
    fn seed_at(&self, index: usize) -> &Seed {
        &self.vault.seeds()[index]
    }

    fn selected_displayed(&self) -> Option<usize> {
        self.table_state.borrow().selected()    
    }

    fn selected_seed_index(&self) -> Option<usize> {
        self.selected_displayed().map(|row| self.displayed[row])
    }

    fn update_displayed(&mut self) {
        let (displayed, default_row) = filter_seeds(self.vault.seeds(), &self.filter.value());
        self.displayed = displayed;
        self.table_state.borrow_mut().select(default_row);
    }
    
    fn move_selected(&mut self, delta: isize, move_content: bool) -> Result<()> {
        let mut table_state = self.table_state.borrow_mut();

        if let Some(prev) = table_state.selected() {
            let new =
                (prev as isize + delta).clamp(0, (self.displayed.len() - 1) as isize) as usize;
                table_state.select(Some(new));

            if move_content {
                self.vault.swap(self.displayed[prev], self.displayed[new])?;
            }
        }
        Ok(())
    }
}

impl State for VaultView {
    fn update(&mut self, term: &mut Terminal, key: KeyCode, modifiers: KeyModifiers) -> Result<Status> {
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);
        let alt = modifiers.contains(KeyModifiers::ALT);

        match key {
            KeyCode::Up => {
                self.move_selected(-1, alt)?;
            }
            KeyCode::Down => {
                self.move_selected(1, alt)?;
            }
            KeyCode::Enter => {
                todo!()
            }
            KeyCode::Char('a') if ctrl => {
                // let mut form = Form::new("Add seed")
                //     .textbox("id", "Identifier", String::default())
                //     .slider("len", "Length", 32, 1, 255, 1)
                //     .slider("salt", "Salt", 0, 0, u64::MAX, 1)
                //     .textbox("chars", "Sets", "ULNSR".to_owned())
                //     .textbox("name", "Username", String::default());
                
                // loop {
                //     if let Some(form_state) = dialog::form(term, Some(self), form)? {
                //         form = form_state;

                //         let identifier = form.get_string("id").unwrap();
                //         if identifier.is_empty() {
                //             dialog::error(term, Some(self), "Identifier must not be empty.")?;
                //             continue;
                //         }
                //         let length = form.get_integer("len").unwrap() as u8;
                //         let salt = form.get_integer("salt").unwrap();
                //         let characters = form.get_string("chars").unwrap();
                //         let username = form.get_string("name").unwrap();
                        
                //         self.vault.push(Seed {
                //             identifier,
                //             length,
                //             salt,
                //             characters: Characters::all(), // TODO
                //             username: if username.is_empty() {
                //                 None
                //             } else {
                //                 Some(username)
                //             },
                //         });
                //         self.update_displayed();
                //     }
                //     break;
                // }
            }
            KeyCode::Char('h') if ctrl => {
                dialog::info(
                    term,
                    Some(self),
                    indoc!(
                        "(alt + ↑/↓)  Move selected seed contents
                         (ctrl + a)   Add new seed
                         (ctrl + r)   Remove selected seed permanently
                         (enter)      Generate password from selected seed"
                    ),
                )?;
            }
            KeyCode::Char('r') if ctrl => {
                if let Some(selected_seed_index) = self.selected_seed_index() {
                    let selected_seed = self.seed_at(selected_seed_index);
                    let confirm_str = format!(
                        "This will permanently remove seed '{}' from the vault. Continue?",
                        selected_seed.identifier
                    );

                    if dialog::confirm(term, Some(self), confirm_str)? {
                        self.vault.remove(selected_seed_index);
                        self.update_displayed();
                    };
                }
            }
            KeyCode::Char(_) if ctrl || alt => (),
            _ => {
                if self.filter.key_down(key, modifiers) {
                    self.update_displayed();
                }
            }
        };
        Ok(Status::Running)
    }
    
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .horizontal_margin(3)
            .vertical_margin(1)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(frame.size());

        // draw the seed table
        {
            let table_widget = Table::new(self.displayed.iter().map(|&seed_index| {
                let Seed {
                    identifier,
                    length,
                    salt,
                    characters,
                    username,
                } = &self.vault.seeds()[seed_index];

                Row::new(vec![
                    identifier.clone(),
                    length.to_string(),
                    salt.to_string(),
                    characters.to_string(),
                    username.clone().unwrap_or_else(|| "None".to_owned()),
                ])
            }))
                .header(
                    Row::new(vec!["NAME", "LENGTH", "SALT", "SETS", "USERNAME"])
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .bottom_margin(1),
                )
                .widths(&[
                    Constraint::Percentage(20),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(50),
                ])
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_stateful_widget(table_widget, layout[0], &mut self.table_state.borrow_mut());
        }

        // draw the filter input box
        {
            let widget = Paragraph::new(self.filter.format(true))
                .block(Block::default().title(" FILTER ").borders(Borders::ALL));
            frame.render_widget(widget, layout[1]);
        }
    }
}

fn hash_vault(vault: &Vault) -> u64 {
    let mut hasher = DefaultHasher::new();
    vault.hash(&mut hasher);
    hasher.finish()
}

fn filter_seeds(seeds: &[Seed], filter: &str) -> (Vec<usize>, Option<usize>) {
    let filtered: Vec<usize> = if filter.is_empty() {
        (0..seeds.len()).collect()
    } else {
        // pair each seed index with it's match score against the filter, removing seeds that don't
        // match at all
        let mut scores: Vec<(usize, isize)> = seeds
            .iter()
            .enumerate()
            .filter_map(|(i, seed)| {
                sublime_fuzzy::best_match(filter, &seed.identifier).map(|m| (i, m.score()))
            })
            .collect();

        // sort pairs such that the highest match score is first, and return the indexes
        scores.sort_by(|(_, a), (_, b)| b.cmp(a));
        scores.into_iter().map(|(i, _)| i).collect()
    };
    let default_row = if filtered.is_empty() { None } else { Some(0) };
    (filtered, default_row)
}
