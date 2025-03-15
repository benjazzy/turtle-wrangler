use std::collections::HashMap;

use crossterm::event::{self, Event, EventStream, KeyCode, KeyEvent};
use futures::{FutureExt, StreamExt};
use ratatui::{
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use tokio::{select, sync::mpsc};
use turtle_types::client_views::TurtleReport;

use crate::{turtle_ticker::TurtleTicker, widgets::TurtleList};

pub enum AppMessage {
    Turtles(Vec<TurtleReport>),
}

#[derive(Debug, Default)]
pub struct App {
    turtles: HashMap<Box<str>, TurtleReport>,
    exit: bool,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let (tx, mut rx) = mpsc::channel(16);
        TurtleTicker::new(tx).start();

        let mut reader = EventStream::new();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut rx, &mut reader).await?;
        }

        Ok(())
    }

    async fn handle_events(
        &mut self,
        message_listener: &mut mpsc::Receiver<AppMessage>,
        event_reader: &mut EventStream,
    ) -> color_eyre::Result<()> {
        let reader = event_reader.next().fuse();

        select! {
            message = message_listener.recv() => {
                if let Some(message) = message {
                    self.handle_message(message)?;
                } else {
                    todo!()
                }
            },
            Some(event) = reader => {
                match event? {
                    Event::Key(key_event) => self.handle_key_event(key_event),
                    _ => {}
                }
            }

        }
        Ok(())
    }

    fn handle_message(&mut self, message: AppMessage) -> color_eyre::Result<()> {
        match message {
            AppMessage::Turtles(turtles) => {
                for turtle in turtles {
                    self.turtles.entry(turtle.name.clone()).insert_entry(turtle);
                }
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // let title = Line::from(" Turtle Tui ".bold());
        // let instructions = Line::from(vec![
        //     " Decrement ".into(),
        //     "<Left>".blue().bold(),
        //     " Increment ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<Q>".blue().bold(),
        // ]);
        // let block = Block::bordered()
        //     .title(title.centered())
        //     .title_bottom(instructions.centered())
        //     .border_set(border::THICK);
        //
        // let turtles = Text::from(
        //     self.turtles
        //         .values()
        //         .fold(String::new(), |mut acc, turtle| {
        //             acc.push_str(&format!("{:?}\n", turtle));
        //
        //             acc
        //         }),
        // );

        let turtle_list = TurtleList::new(self.turtles.values());
        turtle_list.render(area, buf);

        // Paragraph::new(turtle_list)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);
    }
}
