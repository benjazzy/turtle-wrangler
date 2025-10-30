use color_eyre::eyre::eyre;
use futures::future::BoxFuture;
use ratatui::layout::Rect;
use ratatui::prelude::Style;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Block, List, ListItem, ListState};
use ratatui::Frame;

#[derive(Debug, Default)]
pub struct CommandList {
    list_state: ListState,
}

impl CommandList {
    const REBOOT: CommandListItem = CommandListItem {
        name: "Reboot",
        execute: reboot,
    };

    const FORWARD: CommandListItem = CommandListItem {
        name: "Forward",
        execute: forward,
    };

    const BACKWARD: CommandListItem = CommandListItem {
        name: "Backward",
        execute: backward,
    };

    const LEFT: CommandListItem = CommandListItem {
        name: "Left",
        execute: left,
    };

    const RIGHT: CommandListItem = CommandListItem {
        name: "Right",
        execute: right,
    };

    const UP: CommandListItem = CommandListItem {
        name: "Up",
        execute: up,
    };

    const DOWN: CommandListItem = CommandListItem {
        name: "Down",
        execute: down,
    };

    const COMMANDS: &'static [CommandListItem] = &[
        Self::REBOOT,
        Self::FORWARD,
        Self::BACKWARD,
        Self::LEFT,
        Self::RIGHT,
        Self::UP,
        Self::DOWN,
    ];

    pub fn new() -> Self {
        Default::default()
    }

    pub fn select_next(&mut self) {
        self.list_state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.list_state.select_previous();
    }

    pub async fn execute(
        &self,
        turtle_name: &str,
        client: &reqwest::Client,
    ) -> color_eyre::Result<()> {
        let idx = self
            .list_state
            .selected()
            .ok_or_else(|| eyre!("No list item selected"))?;

        (Self::COMMANDS[idx].execute)(turtle_name, client)
            .await
            .map_err(|e| eyre!("reqwest error: {}", e))
            .map(|_| ())
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, selected: bool) {
        let color = if selected {
            Color::Green
        } else {
            Color::default()
        };
        let list = List::new(Self::COMMANDS)
            .block(Block::bordered().title("Commands").border_style(color))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

#[derive(Debug)]
struct CommandListItem {
    name: &'static str,
    execute: fn(&str, &reqwest::Client) -> BoxFuture<'static, reqwest::Result<reqwest::Response>>,
}

impl From<&CommandListItem> for ListItem<'_> {
    fn from(value: &CommandListItem) -> Self {
        value.name.into()
    }
}

fn reboot(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/reboot");
    let fut = client.get(url).send();

    Box::pin(fut)
}

fn forward(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/forward");
    let fut = client.get(url).send();

    Box::pin(fut)
}

fn backward(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/backward");
    let fut = client.get(url).send();

    Box::pin(fut)
}

fn left(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/turn_left");
    let fut = client.get(url).send();

    Box::pin(fut)
}

fn right(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/turn_right");
    let fut = client.get(url).send();

    Box::pin(fut)
}

fn up(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/up");
    let fut = client.get(url).send();

    Box::pin(fut)
}

fn down(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/down");
    let fut = client.get(url).send();

    Box::pin(fut)
}
