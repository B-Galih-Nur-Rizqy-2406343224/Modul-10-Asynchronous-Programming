use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingMessage {
    message_type: String,
    data: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutgoingMessage {
    message_type: String,
    data: Option<String>,
    data_array: Option<Vec<String>>,
}

#[derive(Serialize)]
struct MessageData {
    from: String,
    message: String,
    time: u128,
}

type Users = Arc<Mutex<HashMap<SocketAddr, String>>>;

fn broadcast_users(users: &Users, bcast_tx: &Sender<String>) {
    let user_list: Vec<String> = users.lock().unwrap().values().cloned().collect();
    let msg = OutgoingMessage {
        message_type: "users".to_string(),
        data: None,
        data_array: Some(user_list),
    };
    let _ = bcast_tx.send(serde_json::to_string(&msg).unwrap());
}

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
    users: Users,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) if msg.is_text() => {
                        let text = msg.as_text().unwrap();
                        if let Ok(parsed) = serde_json::from_str::<IncomingMessage>(text) {
                            match parsed.message_type.as_str() {
                                "register" => {
                                    if let Some(username) = parsed.data {
                                        println!("Registered: {username} ({addr})");
                                        users.lock().unwrap().insert(addr, username);
                                        broadcast_users(&users, &bcast_tx);
                                    }
                                }
                                "message" => {
                                    if let Some(content) = parsed.data {
                                        let username = users.lock().unwrap()
                                            .get(&addr).cloned()
                                            .unwrap_or_else(|| addr.to_string());
                                        let time = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap()
                                            .as_millis();
                                        println!("From {username}: {content}");
                                        let message_data = MessageData {
                                            from: username,
                                            message: content,
                                            time,
                                        };
                                        let outgoing = OutgoingMessage {
                                            message_type: "message".to_string(),
                                            data: Some(serde_json::to_string(&message_data).unwrap()),
                                            data_array: None,
                                        };
                                        let _ = bcast_tx.send(serde_json::to_string(&outgoing).unwrap());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => return Err(e.into()),
                    None => break,
                }
            }
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }

    users.lock().unwrap().remove(&addr);
    broadcast_users(&users, &bcast_tx);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = broadcast::channel(16);
    let users: Users = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        let users = users.clone();
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, bcast_tx, users).await
        });
    }
}
