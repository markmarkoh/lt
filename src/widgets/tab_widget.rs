use ratatui::buffer::Buffer;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct TabWidget {
    selected_index: u16
}

impl TabWidget {
    pub fn next(&mut self) {
        self.selected_index += 1;
    }

    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}

impl Widget for &TabWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Tabs::new(["My Issues", "Other Issues"])
            .select(self.selected_index as usize)
            .padding(" ", " ")
            .divider("  ")
            .render(area, buf);
    }
}
