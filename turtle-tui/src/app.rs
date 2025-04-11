mod command_list;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyEventState};
use futures::{FutureExt, StreamExt};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Direction;
use ratatui::{widgets::Widget, DefaultTerminal, Frame};
use std::cell::Cell;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::future::Future;
use std::process::Command;
use reqwest::Client;
use tokio::{select, sync::mpsc};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use turtle_types::client_views::TurtleReport;

use crate::turtle_ticker::TurtleTicker;
use crate::widgets::{CommandLineWidget, TurtleList};
use command_list::CommandList;

pub enum AppMessage {
    Turtles(Vec<TurtleReport>),
}

#[derive(Debug, Default, Eq, PartialEq)]
enum SelectedList {
    #[default]
    TurtleList,
    CommandList,
}

#[derive(Debug, Default)]
pub struct App {
    turtles: HashMap<Box<str>, TurtleReport>,
    selected_list: SelectedList,
    command_list: CommandList,
    exit: bool,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let (tx, mut rx) = mpsc::channel(16);
        let client = Client::new();
        TurtleTicker::new(tx, client.clone()).start();

        let mut reader = EventStream::new();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut rx, &mut reader, &client).await?;
            // dbg!(&self);
        }

        Ok(())
    }

    async fn handle_events(
        &mut self,
        message_listener: &mut mpsc::Receiver<AppMessage>,
        event_reader: &mut EventStream,
        client: &Client
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
                    Event::Key(key_event) if key_event.kind != KeyEventKind::Release => self.handle_key_event(key_event, client).await,
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

    async fn handle_key_event(&mut self, key_event: KeyEvent, client: &Client) {
        match (&self.selected_list, key_event.code) {
            (_, KeyCode::Char('q')) => self.exit(),
            (_, KeyCode::Char('h') | KeyCode::Left) => self.selected_list = SelectedList::TurtleList,
            (_, KeyCode::Char('l') | KeyCode::Right) => self.selected_list = SelectedList::CommandList,
            (SelectedList::CommandList, KeyCode::Char('k') | KeyCode::Up) => self.command_list.select_previous(),
            (SelectedList::CommandList, KeyCode::Char('j') | KeyCode::Down) => self.command_list.select_next(),
            (SelectedList::CommandList, KeyCode::Enter) => self.command_list.execute("Aarika", client).await.unwrap(),
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [turtle_area, cli_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .areas(frame.area());
        
        let [table_area, command_area] = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Min(32), Constraint::Length(32)]).areas(turtle_area);

        let turtle_list = TurtleList::new(self.turtles.values());
        frame.render_widget(turtle_list, table_area);
        
        self.command_list.draw(frame, command_area, self.selected_list == SelectedList::CommandList);
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandLine {
    input: Input,
    history: Vec<Box<str>>,
    history_idx: Cell<usize>,
}

impl CommandLine {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.kind == KeyEventKind::Release {
            return;
        }
        match key_event.code {
            KeyCode::Enter => {
                let command = self.input.value();
                self.add_to_history(command.to_owned());
                self.input.reset();
                // TODO send command
            }
            KeyCode::Up => {
                let Some(text) = self.get_previous_command() else {
                    return;
                };
                self.input = Input::new(text.to_owned());
            }
            KeyCode::Down => {
                let Some(text) = self.get_next_command() else {
                    return;
                };
                self.input = Input::new(text.to_owned());
            }
            _ => {
                self.input.handle_event(&Event::Key(key_event));
            }
        }
    }

    pub fn reset(&mut self) {
        self.input.reset();
    }

    pub fn to_widget(&self, selected: bool) -> CommandLineWidget {
        CommandLineWidget::new(&self.input, selected)
    }

    pub fn get_cursor_x(&self, area: Rect) -> usize {
        let width = (area.width.max(3) - 3) as usize;
        let scroll = self.input.visual_scroll(width);

        self.input.visual_cursor().max(scroll) - scroll + 1
    }

    pub fn add_to_history(&mut self, command: impl Into<Box<str>>) {
        self.history.push(command.into());
        self.history_idx.set(0);
    }

    pub fn get_previous_command(&self) -> Option<&str> {
        let item = self.get_history_command();

        if item.is_some() {
            self.history_idx.set(self.history_idx.get() + 1);
        }

        item
    }

    pub fn get_next_command(&self) -> Option<&str> {
        let item = self.get_history_command();

        self.history_idx
            .set(self.history_idx.get().saturating_sub(1));

        item
    }

    pub fn get_history_command(&self) -> Option<&str> {
        let idx = self.history.len().checked_sub(self.history_idx.get() + 1)?;
        self.history.get(idx).map(|c| c.as_ref())
    }

    pub fn get_history(&self) -> impl Iterator<Item = &str> {
        self.history.iter().rev().map(|c| c.as_ref())
    }
}

