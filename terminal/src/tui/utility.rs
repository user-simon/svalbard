use std::{borrow::Cow, ops::{Add, Sub, Mul, Div}, fmt::Display};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::Widget,
};
use super::input::FormWidget;

/// Trait implemented for all numerical types.
pub trait Number:
    Add<Output=Self> +
    Sub<Output=Self> +
    Mul<Output=Self> +
    Div<Output=Self> +
    Ord + Copy + Display {}
impl<T> Number for T
where T:
    Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> + Ord + Copy + Display
{}

/// Widget wrapper for centering vertically according to specified height.
pub struct CenteredWidget<W: Widget> {
    widget: W,
    height: u16,
}

impl<W: Widget> CenteredWidget<W> {
    pub fn new(widget: W, height: u16) -> Self {
        CenteredWidget { widget, height }
    }
}

impl<W: Widget> Widget for CenteredWidget<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let offset = area
            .height
            .checked_sub(self.height)
            .map(|d| d / 2)
            .unwrap_or(0);
        let centered = Layout::default()
            .constraints([Constraint::Length(offset), Constraint::Length(self.height)])
            .split(area)[1];
        self.widget.render(centered, buf);
    }
}

pub trait Center {
    type W: Widget;
    fn center(self) -> CenteredWidget<Self::W>;
}

/// Utility widget to wrap a string at a certain width. Defined since the wrap functionality in
/// [`Paragraph`](tui::widgets::Paragraph) doesn't allow inspecting the number of produced lines (the "height") for dynamic
/// layouts.
pub struct WrappedString<'a> {
    lines: Vec<Cow<'a, str>>,
    style: Style,
}

impl<'a> WrappedString<'a> {
    pub fn new(string: &'a str, width: u16) -> Self {
        WrappedString {
            lines: textwrap::wrap(&string, width as usize),
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn height(&self) -> u16 {
        self.lines.len() as u16
    }
}

impl<'a> Widget for WrappedString<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, line) in self.lines.iter().enumerate().take(area.height as usize) {
            buf.set_stringn(
                area.x,
                area.y + i as u16,
                line,
                area.width as usize,
                self.style,
            );
        }
    }
}

impl Center for WrappedString<'_> {
    type W = Self;

    fn center(self) -> CenteredWidget<Self> {
        let height = self.height();
        CenteredWidget::new(self, height)
    }
}

impl Center for FormWidget<'_> {
    type W = Self;

    fn center(self) -> CenteredWidget<Self::W> {
        let height = self.0.fields().len() as u16;
        CenteredWidget::new(self, height)
    }
}
