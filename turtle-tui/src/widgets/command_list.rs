use ratatui::widgets::Widget;

pub struct CommandListWidget<'a> {
    commands: &'a [&'a str],
    selected_index: usize,
}

impl<'a> CommandListWidget<'a> {
    pub fn new(commands: &'a [&'a str], selected_index: usize) -> Self {
        CommandListWidget {
            commands,
            selected_index,
        }
    }
}

impl Widget for CommandListWidget<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        todo!()
    }
}
