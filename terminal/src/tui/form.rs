use std::{collections::HashMap, cell::RefCell, iter};

use crossterm::event::{KeyCode, KeyModifiers};
use tui::{text::{Spans, Span}, widgets::{ListState, Widget, List, ListItem, StatefulWidget}, layout::Rect, buffer::Buffer, style::{Style, Color, Modifier}};

use super::{
    input::{StringInput, NumericalInput, Input},
    utility::Number
};

pub trait Field {
    fn title(&self) -> &str;
    fn input(&self) -> &dyn Input;
    fn validate(&self) -> Option<String>;
}

pub struct TextBox {
    input: StringInput,
    title: String,
    validator: Option<fn(&str) -> Option<String>>,
}

impl TextBox {
    pub fn new<S: Into<String>>(title: S) -> Self {
        TextBox {
            input: StringInput::default(),
            title: title.into(),
            validator: None,
        }
    }

    pub fn validator(mut self, validator: fn(&str) -> Option<String>) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn placeholder(mut self, value: String) -> Self {
        self.input.set_value(value);
        self
    }

    pub fn hide(mut self, value: bool) -> Self {
        self.input = self.input.hide(value);
        self
    }

    pub fn value(&self) -> String {
        self.input.value() // TODO cache?
    }
}

impl Field for TextBox {
    fn title(&self) -> &str {
        &self.title
    }

    fn input(&self) -> &dyn Input {
        &self.input
    }

    fn validate(&self) -> Option<String> {
        self.validator.and_then(|f| f(&self.value()))
    }
}

pub struct Slider<T: Number> {
    input: NumericalInput<T>,
    title: String,
    validator: Option<fn(T) -> Option<String>>,
}

impl<T: Number> Slider<T> {
    pub fn new<S: Into<String>>(title: S, default: T, min: T, max: T, step: T) -> Self {
        Self {
            input: NumericalInput::new(default, min, max, step),
            title: title.into(),
            validator: None
        }
    }

    pub fn validator(mut self, validator: fn(T) -> Option<String>) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn value(&self) -> T {
        self.input.value()
    }
}

impl<T: Number> Field for Slider<T> {
    fn title(&self) -> &str {
        &self.title
    }

    fn input(&self) -> &dyn Input {
        &self.input
    }

    fn validate(&self) -> Option<String> {
        self.validator.and_then(|f| f(self.value()))
    }
}

pub struct Form {
    fields: Vec<Box<dyn Field>>,
    title: String,
    list_state: RefCell<ListState>,
    fields_lut: HashMap<&'static str, usize>,
}

impl Form {
    

    fn validate(&self) -> Option<String> {
        self.fields.iter()
            .find_map(|f|
                f.validate().map(|e| format!("{}: {e}", f.title()))
            )
    }

    fn key_down(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        todo!()
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
                    let input_spans = field.input().format(is_selected).0;
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
