mod app;
mod turtle_ticker;
mod widgets;

use app::App;

// #[derive(Debug, Default)]
// struct App {
//     counter: u8,
//     exit: bool,
// }
//
// impl App {
//     pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
//         while !self.exit {
//             terminal.draw(|frame| self.draw(frame))?;
//             self.handle_events()?;
//         }
//
//         Ok(())
//     }
//
//     fn draw(&self, frame: &mut Frame) {
//         frame.render_widget(self, frame.area());
//     }
//
//     fn handle_events(&mut self) -> color_eyre::Result<()> {
//         match event::read()? {
//             Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
//                 self.handle_key_event(key_event)
//             }
//             _ => {}
//         }
//         Ok(())
//     }
//
//     fn handle_key_event(&mut self, key_event: KeyEvent) {
//         match key_event.code {
//             KeyCode::Char('q') => self.exit(),
//             KeyCode::Left => self.decrement_counter(),
//             KeyCode::Right => self.increment_counter(),
//             _ => {}
//         }
//     }
//
//     fn exit(&mut self) {
//         self.exit = true;
//     }
//
//     fn increment_counter(&mut self) {
//         self.counter += 1;
//     }
//
//     fn decrement_counter(&mut self) {
//         self.counter -= 1;
//     }
// }
//
// impl Widget for &App {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let title = Line::from(" Counter App ".bold());
//         let instructions = Line::from(vec![
//             " Decrement ".into(),
//             "<Left>".blue().bold(),
//             " Increment ".into(),
//             "<Right>".blue().bold(),
//             " Quit ".into(),
//             "<Q>".blue().bold(),
//         ]);
//         let block = Block::bordered()
//             .title(title.centered())
//             .title_bottom(instructions.centered())
//             .border_set(border::THICK);
//
//         let counter_text = Text::from(vec![Line::from(vec![
//             "Value: ".into(),
//             self.counter.to_string().yellow(),
//         ])]);
//
//         Paragraph::new(counter_text)
//             .centered()
//             .block(block)
//             .render(area, buf);
//     }
// }

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal).await;
    ratatui::restore();

    result
}
