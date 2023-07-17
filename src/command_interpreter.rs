use crate::db::turtle_operations;
use crate::scheme::{Coordinates, Heading, TurtleType};
use crate::turtle_manager::TurtleManagerHandle;
use crate::turtle_scheme::{RequestType, TurtleCommand};
use sqlx::SqlitePool;
use std::io;
use std::str::FromStr;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::oneshot;
use tracing::{error, info, warn};

fn interpret_command(command: &str) -> Option<TurtleCommand> {
    match command.to_uppercase().as_str() {
        "FORWARD" => Some(TurtleCommand::Forward),
        "BACK" => Some(TurtleCommand::Back),
        "TURNLEFT" => Some(TurtleCommand::TurnLeft),
        "TURNRIGHT" => Some(TurtleCommand::TurnRight),
        "REBOOT" => Some(TurtleCommand::Reboot),
        "INSPECT" => Some(TurtleCommand::Inspect),
        _ => None,
    }
}

fn read_number<F: FromStr>(trimmed_buffer: &str, i: usize) -> Option<F> {
    let c = match trimmed_buffer.split(' ').nth(i) {
        Some(x) => match x.parse::<F>() {
            Ok(x) => x,
            Err(_) => {
                return None;
            }
        },
        None => {
            return None;
        }
    };

    Some(c)
}

fn handle_input(
    trimmed_buffer: &str,
    turtle_manager: TurtleManagerHandle,
    async_handle: &Handle,
    pool: SqlitePool,
) {
    let command = if let Some(c) = trimmed_buffer.chars().next() {
        c
    } else {
        error!("Invalid command");
        return;
    };

    match command.to_ascii_uppercase() {
        'B' => {
            broadcast(trimmed_buffer, async_handle, turtle_manager);
        }
        'C' => {
            send_command(trimmed_buffer, async_handle, turtle_manager);
        }
        'N' => {
            let turtle_name = if let Some(name) = trimmed_buffer.split(' ').nth(1) {
                name.to_string()
            } else {
                error!("Invalid new turtle command missing turtle name");
                return;
            };

            let x: i64 = match read_number(trimmed_buffer, 2) {
                Some(x) => x,
                None => {
                    error!("Invalid x coordinate");
                    return;
                }
            };
            let y: i64 = match read_number(trimmed_buffer, 3) {
                Some(y) => y,
                None => {
                    error!("Invalid y coordinate");
                    return;
                }
            };
            let z: i64 = match read_number(trimmed_buffer, 4) {
                Some(z) => z,
                None => {
                    error!("Invalid z coordinate");
                    return;
                }
            };

            let heading: Heading = match trimmed_buffer.split(' ').nth(5) {
                Some(h) => match Heading::from_str(h) {
                    Some(h) => h,
                    None => {
                        error!("Invalid heading {h}");
                        return;
                    }
                },
                None => {
                    error!("Invalid new turtle command missing heading");
                    return;
                }
            };

            let turtle_type = match trimmed_buffer.split(' ').nth(6) {
                Some(t) => match TurtleType::from_str(t) {
                    Some(t) => t,
                    None => {
                        error!("Invalid turtle type {t}");
                        return;
                    }
                },
                None => {
                    error!("Invalid new turtle command missing turtle type");
                    return;
                }
            };

            async_handle.spawn(async move {
                if let Err(e) = turtle_operations::add_turtle(
                    turtle_name.as_str(),
                    Coordinates { x, y, z },
                    heading,
                    turtle_type,
                    &pool,
                )
                .await
                {
                    error!("Problem creating new turtle {e}");
                }
            });
        }
        'R' => {
            send_request(trimmed_buffer, async_handle, turtle_manager);
        }
        'S' => {
            get_status(async_handle, turtle_manager);
        }
        'P' => {
            set_coordinate(trimmed_buffer, async_handle, turtle_manager);
        }
        'D' => {
            disconnect_turtle(trimmed_buffer, turtle_manager, async_handle);
        }

        c if c != 'Q' => info!("Unknown command"),
        _ => {}
    }
}

fn disconnect_turtle(
    trimmed_buffer: &str,
    turtle_manager: TurtleManagerHandle,
    async_handle: &Handle,
) {
    let name = match trimmed_buffer.split_whitespace().nth(1) {
        Some(n) => n.to_string(),
        None => {
            error!("Invalid disconnect command entered");
            return;
        }
    };

    async_handle.spawn(async move { turtle_manager.disconnect(name).await });
}

fn set_coordinate(
    trimmed_buffer: &str,
    async_handle: &Handle,
    turtle_manager: TurtleManagerHandle,
) {
    let turtle_name = match trimmed_buffer.split_whitespace().nth(1) {
        Some(n) => n.to_string(),
        None => {
            error!("Invalid name");
            return;
        }
    };

    let option = match trimmed_buffer.split_whitespace().nth(2) {
        Some(o) => match o.to_uppercase().as_str() {
            "X" => |current_position: Coordinates, x: i64| Coordinates {
                x,
                ..current_position
            },
            "Y" => |current_position: Coordinates, y: i64| Coordinates {
                y,
                ..current_position
            },
            "Z" => |current_position: Coordinates, z: i64| Coordinates {
                z,
                ..current_position
            },
            _ => {
                error!("Unknown option {o}");
                return;
            }
        },
        None => {
            error!("Invalid option");
            return;
        }
    };

    let coordinate = match trimmed_buffer.split_whitespace().nth(3) {
        Some(c) => match c.parse::<i64>() {
            Ok(c) => c,
            Err(_) => {
                error!("Invalid coordinate {c}");
                return;
            }
        },
        None => {
            error!("Invalid coordinate");
            return;
        }
    };

    async_handle.spawn(async move {
        let turtle = match turtle_manager.get_turtle(&turtle_name).await {
            Some(t) => t,
            None => {
                error!("Unknown turtle {turtle_name}");
                return;
            }
        };

        let current_position = match turtle.get_db().get_coordinates().await {
            Some(p) => p,
            None => Coordinates { x: 0, y: 0, z: 0 },
        };

        let new_position = option(current_position, coordinate);
        if let Err(e) = turtle.get_db().set_coordinates(new_position).await {
            error!("Problem setting turtle position: {e}");
        };
    });
}

fn get_status(async_handle: &Handle, turtle_manager: TurtleManagerHandle) {
    async_handle.spawn(async move {
        println!(
            "{}",
            turtle_manager
                .get_status()
                .await
                .unwrap_or("Problem getting status".to_string())
        );
    });
}

fn send_request(trimmed_buffer: &str, async_handle: &Handle, turtle_manager: TurtleManagerHandle) {
    let turtle_name = if let Some(name) = trimmed_buffer.split(' ').nth(1) {
        name.to_string()
    } else {
        error!("Invalid run command missing turtle name");
        return;
    };
    let request = if let Some(request) = trimmed_buffer.split(' ').nth(2) {
        match request.to_uppercase().as_str() {
            "INSPECT" => RequestType::Inspect,
            "PING" => RequestType::Ping,
            _ => {
                error!("Invalid request {request}");
                return;
            }
        }
    } else {
        error!("Invalid run command missing command");
        return;
    };

    async_handle.spawn(async move {
        let try_turtle = turtle_manager.get_turtle(turtle_name.clone()).await;
        if let Some(turtle) = try_turtle {
            match tokio::time::timeout(Duration::from_secs(10), turtle.request(request)).await {
                Ok(Ok(response)) => {
                    info!("Got response from {turtle_name}: {:?}", response)
                }
                Err(_) => error!("Timeout getting response from {turtle_name}"),
                Ok(Err(_)) => error!("Problem getting response form {turtle_name}"),
            }
        }
    });
}

fn send_command(trimmed_buffer: &str, async_handle: &Handle, turtle_manager: TurtleManagerHandle) {
    let turtle_name = if let Some(name) = trimmed_buffer.split(' ').nth(1) {
        name.to_string()
    } else {
        error!("Invalid run command missing turtle name");
        return;
    };
    let command = if let Some(command) = trimmed_buffer.split(' ').nth(2) {
        if let Some(command) = interpret_command(command) {
            command
        } else {
            error!("Unknown turtle command {command}");
            return;
        }
    } else {
        error!("Invalid run command missing command");
        return;
    };

    async_handle.spawn(async move {
        let try_turtle = turtle_manager.get_turtle(turtle_name).await;
        if let Some(turtle) = try_turtle {
            if let Err(e) = turtle.send(command).await {
                warn!("Could not send command to turtle {e}");
            }
        }
    });
}

fn broadcast(trimmed_buffer: &str, async_handle: &Handle, turtle_manager: TurtleManagerHandle) {
    let turtle_command_string = if let Some(c) = trimmed_buffer.split(' ').nth(1) {
        c.to_string()
    } else {
        error!("Invalid command to run");
        return;
    };

    let turtle_command = if let Some(command) = interpret_command(turtle_command_string.as_str()) {
        command
    } else {
        error!("Unknown command {turtle_command_string}");
        return;
    };

    async_handle.spawn(async move { turtle_manager.broadcast(turtle_command).await });
}

pub fn read_input(
    close_tx: oneshot::Sender<()>,
    turtle_manager: TurtleManagerHandle,
    async_handle: Handle,
    pool: SqlitePool,
) {
    let mut buffer = String::new();
    while buffer.to_uppercase() != "Q" {
        buffer.clear();

        io::stdin().read_line(&mut buffer).unwrap();
        let trimmed_buffer = buffer.trim_end();

        handle_input(
            trimmed_buffer,
            turtle_manager.clone(),
            &async_handle,
            pool.clone(),
        );
    }

    close_tx.send(()).unwrap();
}
