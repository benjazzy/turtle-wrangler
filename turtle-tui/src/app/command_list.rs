use std::future::Future;

use futures::future::BoxFuture;

pub struct CommandList {
    client: reqwest::Client,
    command_index: usize,
}

impl CommandList {
    const REBOOT: CommandListItem = CommandListItem {
        name: "Reboot",
        execute: reboot,
    };
    const COMMANDS: &[CommandListItem] = &[Self::REBOOT];

    pub fn new(client: reqwest::Client) -> Self {
        CommandList {
            client,
            command_index: 0,
        }
    }

    pub fn select_next(&mut self) {
        self.command_index += 1;

        if self.command_index >= Self::COMMANDS.len() {
            self.command_index = Self::COMMANDS.len() - 1;
        }
    }

    pub fn select_previous(&mut self) {
        self.command_index = self.command_index.saturating_sub(1);
    }

    pub async fn execute(
        &self,
        turtle_name: &str,
        client: &reqwest::Client,
    ) -> reqwest::Result<reqwest::Response> {
        (Self::COMMANDS[self.command_index].execute)(turtle_name, client).await
    }
}

struct CommandListItem {
    name: &'static str,
    execute: fn(&str, &reqwest::Client) -> BoxFuture<'static, reqwest::Result<reqwest::Response>>,
}

fn reboot(
    turtle_name: &str,
    client: &reqwest::Client,
) -> BoxFuture<'static, reqwest::Result<reqwest::Response>> {
    let url = format!("http://localhost:8080/turtle/{turtle_name}/reboot");
    let fut = client.get(url).send();

    Box::pin(fut)
}
