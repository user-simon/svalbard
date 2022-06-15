use crossterm::event::{KeyCode, KeyModifiers};
use std::{collections::HashMap, iter, cell::RefCell};
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{List, ListItem, ListState, StatefulWidget, Widget},
};

use super::utility::Number;

pub trait Input {
    /// Returns whether value changed.
    fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool;
    fn format(&self, selected: bool) -> Spans;
}

/// Utility to handle string input.
pub struct StringInput {
    /// Contains the entered text. Not defined as [`String`] to allow easier index-based operations.
    value: Vec<char>,
    /// Specifies the position of the caret.
    caret: usize,
    /// Enables a second mode of operation where user input is hidden, e.g. for password inputs.
    hidden: bool,
}

impl StringInput {
    pub fn hide(mut self, hide: bool) -> Self {
        self.hidden = hide;
        self
    }

    pub fn value(&self) -> String {
        self.value.iter().collect()
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value.chars().collect();
        self.caret = value.len();
    }

    /// Determines the index to jump to in specified direction e.g. when `CTRL + ←/→` is pressed.
    fn jump_point(&self, left: bool) -> usize {
        // determine the direction to move in and index to stop at
        let (dir, stop): (isize, usize) = if left { (-1, 0) } else { (1, self.value.len()) };

        // skip search if content is hidden or we cannot move further in specified direction
        if self.caret == stop || self.hidden {
            return stop;
        }

        // utility for determining whether an index is a jump point
        let is_jump_point = |i: usize| {
            let current = self.value[i];
            let left = self.value[i - 1];

            let is_symbol = |c: char| !c.is_whitespace() && !c.is_alphanumeric();

            is_symbol(current)        && !is_symbol(left) ||
            current.is_alphanumeric() && !left.is_alphanumeric()
        };

        // iterate over all indices in range `(caret..stop)` until a jump point is found
        let mut i = self.caret;
        loop {
            i = ((i as isize) + dir) as usize;

            if i == stop || is_jump_point(i) {
                break i;
            };
        }
    }
}

impl Default for StringInput {
    fn default() -> Self {
        Self {
            value: Vec::default(),
            caret: 0,
            hidden: false
        }
    }
}

impl Input for StringInput {
    fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);

        match (key, ctrl) {
            (KeyCode::Left, false) => {
                self.caret = self.caret.checked_sub(1).unwrap_or(0);
                false
            }
            (KeyCode::Left, true) => {
                self.caret = self.jump_point(true);
                false
            }
            (KeyCode::Right, false) => {
                self.caret = (self.caret + 1).min(self.value.len());
                false
            }
            (KeyCode::Right, true) => {
                self.caret = self.jump_point(false);
                false
            }
            (KeyCode::Backspace, false) => {
                if self.caret > 0 {
                    self.caret -= 1;
                    self.value.remove(self.caret);
                    true
                } else {
                    false
                }
            }
            (KeyCode::Backspace, true) => {
                if self.caret > 0 {
                    let end = self.jump_point(true);
                    self.value.drain(end..self.caret);
                    self.caret = end;
                    true
                } else {
                    false
                }
            }
            (KeyCode::Delete, false) => {
                if self.caret < self.value.len() {
                    self.value.remove(self.caret);
                    true
                } else {
                    false
                }
            }
            (KeyCode::Delete, true) => {
                if self.caret < self.value.len() {
                    let end = self.jump_point(false);
                    self.value.drain(self.caret..end);
                    true
                } else {
                    false
                }
            }
            (KeyCode::Char(char), _) => {
                self.value.insert(self.caret, char);
                self.caret += 1;
                true
            }
            _ => false,
        }
    }

    fn format(&self, with_caret: bool) -> Spans {
        let mut content: Vec<char> = if self.hidden {
            iter::repeat('•').take(self.value.len()).collect()
        } else {
            self.value.iter().cloned().collect()
        };
        
        let spans = if with_caret {
            content.push(' ');

            let [pre, caret, post] = {
                let (a, b) = content.split_at(self.caret);
                let (b, c) = b.split_at(1);
                [a, b, c].map(|chars| String::from_iter(chars))
            };
            vec![
                Span::raw(pre),
                Span::styled(caret, Style::default().add_modifier(Modifier::REVERSED)),
                Span::raw(post),
            ]
        } else {
            vec![Span::raw(String::from_iter(content))]
        };
        Spans::from(spans)
    }
}

/// Utility to handle integral input.
pub struct NumericalInput<T: Number> {
    default: T,
    value: T,
    min: T,
    max: T,
    step: T,
}

impl<T: Number> NumericalInput<T> {
    pub fn new(default: T, min: T, max: T, step: T) -> Self {
        debug_assert!(default >= min && default <= max);

        NumericalInput {
            default: default,
            value: default,
            min,
            max,
            step,
        }
    }

    pub fn set_value(&mut self, value: T) {
        debug_assert!(value >= self.min && value <= self.max);
        self.value = value;
    }

    pub fn value(&self) -> T {
        self.value
    }
}

impl<T: Number> Input for NumericalInput<T> {
    fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);
        let before = self.value;

        match (key, ctrl) {
            (KeyCode::Left, false) => {
                self.value = if self.value >= self.min + self.step {
                    self.value - self.step
                } else {
                    self.min
                };
            }
            (KeyCode::Left, true) => {
                self.value = if self.value <= self.default {
                    self.min
                } else {
                    self.default
                }
            }
            (KeyCode::Right, false) => {
                self.value = if self.value <= self.max - self.step {
                    self.value + self.step
                } else {
                    self.max
                };
            }
            (KeyCode::Right, true) => {
                self.value = if self.value >= self.default {
                    self.max
                } else {
                    self.default
                }
            }
            _ => (),
        };
        self.value != before
    }

    fn format(&self, _selected: bool) -> Spans {
        Spans::from(vec![Span::from(format!("<{}>", self.value))])
    }
}

enum InputType {
    String(StringInput),
    Integer(NumericalInput<u64>),
}

pub struct Field {
    key: &'static str,
    title: String,
    input_type: InputType,
}

impl Field {
    fn title(&self) -> &str {
        &self.title
    }

    fn format_input(&self, as_selected: bool) -> Spans {
        match &self.input_type {
            InputType::String(input) => input.format(as_selected),
            InputType::Integer(input) => input.format(as_selected),
        }
    }

    fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match &mut self.input_type {
            InputType::String(input) => input.key_down(key, modifiers),
            InputType::Integer(input) => input.key_down(key, modifiers),
        };
    }
}

pub struct Form {
    title: String,
    fields: Vec<Field>,
    list_state: RefCell<ListState>,
    fields_lut: HashMap<&'static str, usize>,
}

impl Form {
    pub fn new<S: Into<String>>(title: S) -> Self {
        Form {
            title: title.into(),
            fields: Vec::new(),
            list_state: RefCell::default(),
            fields_lut: HashMap::new(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Up => self.move_selected(-1),
            KeyCode::Down => self.move_selected(1),
            _ => {
                if let Some(selected) = self.list_state.borrow().selected() {
                    self.fields[selected].key_down(key, modifiers)
                }
            }
        }
    }

    pub fn textbox<S: Into<String>>(self, key: &'static str, title: S) -> Self {
        self.add(Field {
            key,
            title: title.into(),
            input_type: InputType::String(StringInput::default()),
        })
    }

    pub fn password<S: Into<String>>(self, key: &'static str, title: S) -> Self {
        self.add(Field {
            key,
            title: title.into(),
            input_type: InputType::String(StringInput::default().hide(true)),
        })
    }

    pub fn slider<S: Into<String>>(self, key: &'static str, title: S, default: u64, min: u64, max: u64, step: u64) -> Self {
        self.add(Field {
            key,
            title: title.into(),
            input_type: InputType::Integer(NumericalInput::new(default, min, max, step)),
        })
    }

    pub fn checkbox<S: Into<String>>(self, key: &'static str, title: S, default: bool) -> Self {
        self.slider(key, title, default as u64, 0, 1, 1)
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn get_field(&self, key: &str) -> Option<&Field> {
        self.fields_lut.get(key).map(|&i| &self.fields[i])
    }

    pub fn get_integer(&self, key: &str) -> Option<u64> {
        match self.get_field(key) {
            Some(Field {
                input_type: InputType::Integer(input),
                ..
            }) => Some(input.value()),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_integer(key).map(|value| value != 0)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.get_field(key) {
            Some(Field {
                input_type: InputType::String(input),
                ..
            }) => Some(input.value()),
            _ => None,
        }
    }

    fn add(mut self, field: Field) -> Self {
        {
            let mut list_state = self.list_state.borrow_mut();
    
            if list_state.selected().is_none() {
                list_state.select(Some(0))
            }
            self.fields_lut.insert(field.key, self.fields.len());
            self.fields.push(field);
        }
        self
    }
    
    fn move_selected(&mut self, delta: isize) {
        let mut list_state = self.list_state.borrow_mut();
        
        if let Some(prev) = list_state.selected() {
            let new = (prev as isize + delta).clamp(0, (self.fields.len() - 1) as isize) as usize;
            list_state.select(Some(new));
        }
    }
}

pub struct FormWidget<'a>(pub &'a Form);

impl<'a> Widget for FormWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let form = self.0;
        let max_title_len = form
            .fields
            .iter()
            .map(|field| field.title().len())
            .max()
            .unwrap_or(0);
        let list_widget = List::new(
            form.fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let is_selected = form
                        .list_state
                        .borrow()
                        .selected()
                        .map(|selected| i == selected)
                        .unwrap_or(false);
                    let title = field.title();
                    let padding_str: String = iter::repeat(' ')
                        .take(max_title_len - title.len())
                        .collect();
                    let input_spans = field.format_input(is_selected).0;
                    let mut spans = Vec::with_capacity(input_spans.len() + 1);
                    spans.extend([Span::raw(format!("{padding_str}{title}: "))]);
                    spans.extend(input_spans.into_iter());
                    ListItem::new(Spans::from(spans))
                })
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
        StatefulWidget::render(list_widget, area, buf, &mut form.list_state.borrow_mut());
    }
}
