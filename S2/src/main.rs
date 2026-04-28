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
    // Shoot,
    // EndMatch,
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

                if let Err(e) = send_command(
                    &mut write,
                    WebSocketMessage {
                        command: Command::Move,
                        args: serde_json::json!({"hero_id":turn_args.state.heroes[0].id, "x":turn_args.state.heroes[0].x, "y":turn_args.state.heroes[0].y+1}),
                    },
                )
                .await
                {
                    println!("Failed to send Move command: {e}");
                    break;
                }

                if let Err(e) = send_command(
                    &mut write,
                    WebSocketMessage {
                        command: Command::Move,
                        args: serde_json::json!({"hero_id":turn_args.state.heroes[1].id, "x":turn_args.state.heroes[1].x, "y":turn_args.state.heroes[1].y+1}),
                    },
                )
                .await
                {
                    println!("Failed to send Move command: {e}");
                    break;
                }
            }
            Command::Move => {
                panic!("Server is sending Move");
            }
        }
    }
}
