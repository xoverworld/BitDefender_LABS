use anyhow::Context;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::sync::{Arc, Mutex}; // ADDED THIS
use tokio::net::TcpListener; // ADDED
use tokio::sync::broadcast;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, accept_async, connect_async, tungstenite::Message,
}; // ADDED

use crate::helper::calculate_score;
mod astar;
mod helper;
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
    let (tx, _rx) = broadcast::channel::<String>(500);
    let tx_clone = tx.clone();

    // --- ADDED: A shared cache to store all turns ---
    let match_history = Arc::new(Mutex::new(Vec::<String>::new()));
    let history_for_server = match_history.clone();
    // ------------------------------------------------

    // Spawn a background task to host the local WebSocket server
    tokio::spawn(async move {
        let listener = loop {
            match TcpListener::bind("127.0.0.1:9001").await {
                Ok(l) => break l,
                Err(e) => {
                    println!("Visualizer port 9001 is busy ({}). Retrying...", e);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        };

        println!("Local visualizer server listening on ws://127.0.0.1:9001");

        while let Ok((stream, _)) = listener.accept().await {
            match accept_async(stream).await {
                Ok(mut ws_stream) => {
                    println!("Web visualizer connected!");

                    // --- NEW LOGIC: Send the past history immediately ---
                    let past_turns = {
                        let lock = history_for_server.lock().unwrap();
                        lock.clone()
                    };

                    for turn_json in past_turns {
                        let _ = ws_stream.send(Message::Text(turn_json.into())).await;
                    }
                    // ----------------------------------------------------

                    let mut rx = tx_clone.subscribe();

                    // Listen for live updates
                    tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await {
                            if ws_stream.send(Message::Text(msg.into())).await.is_err() {
                                println!("Web visualizer disconnected.");
                                break;
                            }
                        }
                    });
                }
                Err(e) => println!("Error during local websocket handshake: {}", e),
            }
        }
    });

    let url = "wss://bitdefenders.cvjd.me/ws";
    let (ws, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws.split();
    let mut path_matrix: [[i32; 51]; 90] = [[1; 51]; 90];
    let mut score_matrix: [[i32; 51]; 90] = [[0; 51]; 90];
    println!("connected");

    while let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        let message: WebSocketMessage = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        println!("{message:?}");

        let start_game_args: protocol::StartMatchArgs;
        let turn_args: protocol::StartTurnArgs;
        let mut my_id: i32 = 0;
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
                my_id = start_game_args.your_player_id;

                helper::initialize_walls(&mut path_matrix, &start_game_args.state.walls);
                helper::initialize_walls(&mut score_matrix, &start_game_args.state.walls);
                // score_matrix[(25) as usize][(88) as usize] = 10;
            }
            Command::StartTurn => {
                println!("Turn started");
                turn_args = serde_json::from_value(message.args).unwrap();
                let mut enemies: Vec<protocol::Hero> = Vec::new();
                let mut projectiles: Vec<protocol::Projectile> = Vec::new();
                for hero in turn_args.state.heroes.iter() {
                    if hero.owner_id != my_id {
                        enemies.push(hero.clone());
                    }
                }
                for projectile in turn_args.state.projectiles.iter() {
                    if projectile.owner_id != my_id {
                        projectiles.push(projectile.clone());
                    }
                }
                calculate_score(&mut score_matrix, &path_matrix, enemies, projectiles);

                // --- BROADCAST TO WEB VIEWER ---
                // 1. Extract coordinates for rendering on top of the matrix

                let mut my_heroes_pos = vec![];
                let mut enemy_heroes_pos = vec![];
                for hero in &turn_args.state.heroes {
                    if hero.owner_id == my_id {
                        my_heroes_pos.push((hero.x, hero.y));
                    } else {
                        enemy_heroes_pos.push((hero.x, hero.y));
                    }
                }

                let mut proj_pos = vec![];
                for p in &turn_args.state.projectiles {
                    proj_pos.push((p.x, p.y));
                }

                // 2. Prepare the matrix
                let serializable_matrix: Vec<&[i32]> =
                    score_matrix.iter().map(|row| row.as_slice()).collect();

                // 3. Package it all into a single JSON payload
                let debug_payload = serde_json::json!({
                    "turn": turn_args.turn,
                    "matrix": serializable_matrix,
                    "my_heroes": my_heroes_pos,
                    "enemy_heroes": enemy_heroes_pos,
                    "projectiles": proj_pos
                });

                if let Ok(json_str) = serde_json::to_string(&debug_payload) {
                    // --- NEW: Save to history for late connections ---
                    if let Ok(mut history) = match_history.lock() {
                        history.push(json_str.clone());
                    }
                    // Broadcast to live connections
                    let _ = tx.send(json_str);
                }
                // -------------------------------
                //

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
                // for wall in turn_args.state.walls {
                // if wall.x == x1 && wall.y == y1 + 3 {
                //     args1 = serde_json::json!({"hero_id":turn_args.state.heroes[0].id, "x":x1+3,"y":y1})
                // }
                let (next_y, next_x) =
                    helper::next_move(x1 as usize, y1 as usize, &path_matrix, &score_matrix);
                // println!("{} {}", next_x, next_y);
                args1 = serde_json::json!({"hero_id":turn_args.state.heroes[0].id, "x":next_x,"y":next_y});
                if turn_args.state.heroes.len() > 1 && turn_args.state.heroes[1].owner_id == my_id
                // && wall.y == y2 + 3
                // && wall.x == x2
                {
                    let (next_y, next_x) =
                        helper::next_move(x2 as usize, y2 as usize, &path_matrix, &score_matrix);
                    args2 = serde_json::json!({"hero_id":turn_args.state.heroes[1].id, "x":next_x,"y":next_y});
                    // args2 = serde_json::json!({"hero_id":turn_args.state.heroes[1].id, "x":x2+3, "y":y2});
                }
                // }
                if turn_args.state.heroes.len() > 2 {
                    if turn_args.state.heroes[0].cooldown == 0 {
                        command1 = Command::Shoot;
                        args1 = serde_json::json!({"hero_id":turn_args.state.heroes[0].id, "x":turn_args.state.heroes[2].x,"y":turn_args.state.heroes[2].y});
                    }
                    if turn_args.state.heroes[1].owner_id == my_id
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

                if turn_args.state.heroes.len() > 1 && turn_args.state.heroes[1].owner_id == my_id {
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

    // --- ADD THIS TO THE VERY END OF YOUR MAIN FUNCTION ---
    println!("---");
    println!("Game connection closed. The match is over.");
    println!("The visualizer server is STILL RUNNING at ws://127.0.0.1:9001");
    println!("You can refresh your browser to view the history.");
    println!("Press Ctrl+C to kill the bot.");

    // This keeps the program (and your background debug server) alive forever
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
