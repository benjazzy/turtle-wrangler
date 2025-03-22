use color_eyre::owo_colors::OwoColorize;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph, Widget};
use tui_input::Input;

#[derive(Debug, Clone)]
pub struct CommandLineWidget<'a> {
    input: &'a Input,
    selected: bool,
}

impl<'a> CommandLineWidget<'a> {
    pub fn new(input: &'a Input, selected: bool) -> Self {
        Self { input, selected }
    }
}

impl Widget for CommandLineWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let color = if self.selected {
            Color::Green
        } else {
            Color::default()
        };
        let block = Block::bordered().style(color);
        let paragraph = Paragraph::new(self.input.value())
            .block(block)
            .scroll((0, scroll as u16));

        paragraph.render(area, buf);
    }
}
