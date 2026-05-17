use anyhow::Context;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
mod protocol;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    command: Command,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Command {
    Hello,
    Login,
    Error,
    Ready,
    // Challenge,
    Practice,
    StartMatch,
    StartTurn,
    Move,
    Shoot,
    EndMatch,
}

async fn send_command<
    S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
>(
    write: &mut S,
    msg: WebSocketMessage,
) -> anyhow::Result<()> {
    let msg_deserialized = serde_json::to_string(&msg).context("serialize message")?;
    write
        .send(Message::Text(msg_deserialized.into()))
        .await
        .context("send message")?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let url = "wss://bitdefenders.cvjd.me/ws";
    let (ws, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws.split();

    println!("connected");

    while let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        let message: WebSocketMessage = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        println!("{message:?}");

        let mut start_game_args: protocol::StartMatchArgs;
        let mut turn_args: protocol::StartTurnArgs;
        match message.command {
            Command::Hello => {
                // Send login
                if let Err(e) = send_command(
                    &mut write,
                    WebSocketMessage {
                        command: Command::Login,
                        args: serde_json::json!({"version": 1, "name": "Erik"}),
                    },
                )
                .await
                {
                    println!("Failed to send login command: {e}");
                    break;
                }
            }
            Command::Login => {
                panic!("What are you doing here?");
            }
            Command::Error => {
                println!("Error: {message:?}");
                break;
            }
            Command::Ready => {
                println!("You are ready to play!");
                if let Err(e) = send_command(
                    &mut write,
                    WebSocketMessage {
                        command: Command::Practice,
                        args: serde_json::json!({}),
                    },
                )
                .await
                {
                    println!("Failed to send Practice/Challange command: {e}");
                    break;
                }
            }
            Command::Practice => {
                panic!("Server is sending Ready");
            }
            Command::StartMatch => {
                println!("Start match");
                start_game_args = serde_json::from_value(message.args).unwrap();
            }
            Command::StartTurn => {
                println!("Turn started");
                turn_args = serde_json::from_value(message.args).unwrap();
                let x1 = turn_args.state.heroes[0].x;
                let y1 = turn_args.state.heroes[0].y;

                let mut x2 = 0;
                let mut y2 = 0;

                let mut command1: Command = Command::Move;
                let mut command2: Command = Command::Move;

                let mut args1 =
                    serde_json::json!({"hero_id":turn_args.state.heroes[0].id, "x":x1,"y":y1+3});
                let mut args2 = serde_json::json!({});
                if turn_args.state.heroes.len() > 1 {
                    x2 = turn_args.state.heroes[1].x;
                    y2 = turn_args.state.heroes[1].y;
                    args2 = serde_json::json!({"hero_id":turn_args.state.heroes[1].id, "x":x2, "y":y2+3});
                }
                for wall in turn_args.state.walls {
                    if wall.x == x1 && wall.y == y1 + 3 {
                        args1 = serde_json::json!({"hero_id":turn_args.state.heroes[0].id, "x":x1+3,"y":y1})
                    }
                    if turn_args.state.heroes.len() > 1
                        && turn_args.state.heroes[1].owner_id == 0
                        && wall.y == y2 + 3
                        && wall.x == x2
                    {
                        args2 = serde_json::json!({"hero_id":turn_args.state.heroes[1].id, "x":x2+3, "y":y2});
                    }
                }
                if turn_args.state.heroes.len() > 2 {
                    if turn_args.state.heroes[0].cooldown == 0 {
                        command1 = Command::Shoot;
                        args1 = serde_json::json!({"hero_id":turn_args.state.heroes[0].id, "x":turn_args.state.heroes[2].x,"y":turn_args.state.heroes[2].y});
                    }
                    if turn_args.state.heroes[1].owner_id == 0
                        && turn_args.state.heroes[1].cooldown == 0
                    {
                        command2 = Command::Shoot;
                        args2 = serde_json::json!({"hero_id":turn_args.state.heroes[1].id, "x":turn_args.state.heroes[2].x,"y":turn_args.state.heroes[2].y});
                    }
                }

                if let Err(e) = send_command(
                    &mut write,
                    WebSocketMessage {
                        command: command1,
                        args: args1,
                    },
                )
                .await
                {
                    println!("Failed to send Move command: {e}");
                    break;
                }

                if turn_args.state.heroes.len() > 1 && turn_args.state.heroes[1].owner_id == 0 {
                    if let Err(e) = send_command(
                        &mut write,
                        WebSocketMessage {
                            command: command2,
                            args: args2,
                        },
                    )
                    .await
                    {
                        println!("Failed to send Move command: {e}");
                        break;
                    }
                }
            }
            Command::Move => {
                panic!("Server is sending Move");
            }
            Command::Shoot => {
                panic!("Server is sending Shoot");
            }
            Command::EndMatch => {
                println!("Match has concluded!");
            }
        }
    }
}
