use std::{
    collections::HashMap,
    env,
    hash::Hash,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use ratatui::style::Color;
use tokio::net::{TcpListener, TcpStream};

use tokio_tungstenite::tungstenite::protocol::Message;

struct UserData {
    color: Color,
    user_name: String,
}
struct User {
    user_data: Arc<RwLock<UserData>>,
    tx: Tx,
}

type Tx = UnboundedSender<Message>;

type PeerMap = Arc<Mutex<HashMap<SocketAddr, User>>>;

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let mut new_user_data = common::NewUserSet {
        user_name: "ERROR".into(),
        color: Color::Red,
    };
    let (outgoing, mut incoming) = ws_stream.split();
    while let Some(data) = incoming.next().await {
        let msg = data.unwrap();
        match serde_json::from_slice::<common::NewUserSet>(&msg.into_data()) {
            Ok(user_data) => {
                new_user_data = user_data;
                break;
            }
            Err(_) => {}
        }
    }

    let msg = common::Message {
        user_name: "SERVER".into(),
        content: format!("connected succesfully as '{}'", new_user_data.user_name).into(),
        color: ratatui::style::Color::Cyan,
    }
    .serialize();

    let (tx, rx) = unbounded();
    tx.unbounded_send(Message::Binary(msg)).unwrap();

    let cur_user_data = Arc::new(RwLock::new(UserData {
        color: new_user_data.color,
        user_name: new_user_data.user_name.into(),
    }));

    peer_map.lock().unwrap().insert(
        addr,
        User {
            user_data: Arc::clone(&cur_user_data),
            tx,
        },
    );

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let msg = msg.to_text().unwrap().trim();
        println!("Received a message from {}: {}", addr, msg);
        if msg.starts_with('/') && msg.len() > 1 {
            let peers = peer_map.lock().unwrap();
            let cmd = msg[1..].to_lowercase();
            let args = cmd.splitn(3, ' ').collect::<Vec<&str>>();
            let response: common::Message = match args[..] {
                ["setcolor", new_color] => handle_new_color(new_color, &cur_user_data),
                ["setname", new_name] => handle_change_username(new_name, &peers, &cur_user_data),
                ["whisper" | "msg" | "w", username, message] => {
                    handle_whisper(&cur_user_data, &peers, username, message)
                }
                _ => common::Message::error("Invalid Command"),
            };
            peers
                .get(&addr)
                .unwrap()
                .tx
                .unbounded_send(Message::Binary(response.serialize()))
                .unwrap();
        } else {
            let peers = peer_map.lock().unwrap();

            let broadcast_recipients = peers.values().map(|user| &user.tx);

            let user_data = cur_user_data.read().unwrap();

            let msg_to_send = common::Message {
                user_name: user_data.user_name.clone().into(),
                content: msg.into(),
                color: user_data.color,
            };
            println!("Sending Message {:?}", msg_to_send);
            let msg_to_send = msg_to_send.serialize();

            for recp in broadcast_recipients {
                recp.unbounded_send(Message::Binary(msg_to_send.clone()))
                    .unwrap();
            }
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

fn handle_whisper<'a>(
    cur_user_data: &RwLock<UserData>,
    peers: &HashMap<SocketAddr, User>,
    username: &'a str,
    message: &'a str,
) -> common::Message<'a> {
    let cur_user_data = cur_user_data.read().unwrap();
    peers
        .values()
        .find(|user| user.user_data.read().unwrap().user_name == username)
        .and_then(|user| {
            user.tx
                .unbounded_send(Message::Binary(
                    common::Message {
                        user_name: format!("Whisper from '{}'", cur_user_data.user_name).into(),
                        content: message.into(),
                        color: Color::Magenta,
                    }
                    .serialize(),
                ))
                .ok()
        })
        .map_or_else(
            || common::Message::error(&format!("No user named: {}", username)),
            |_| common::Message {
                user_name: format!("Whispered to '{}' message", username).into(),
                content: message.into(),
                color: Color::Magenta,
            },
        )
}

fn handle_new_color<'a>(
    new_color: &'a str,
    cur_user_data: &RwLock<UserData>,
) -> common::Message<'a> {
    new_color.parse::<Color>().map_or_else(
        |_| common::Message::error(&format!("Invalid Color: '{}'", new_color)),
        |color| {
            cur_user_data.write().unwrap().color = color;
            common::Message::server(&format!("Set Color to {:?} succesfully!", color))
        },
    )
}

fn handle_change_username<'a>(
    new_name: &'a str,
    peers: &HashMap<SocketAddr, User>,
    cur_user_data: &RwLock<UserData>,
) -> common::Message<'a> {
    (new_name.is_ascii() && new_name.len() < 15)
        .then(|| {
            (!user_name_exists(peers, new_name))
                .then(|| {
                    cur_user_data.write().unwrap().user_name = new_name.to_string();
                    common::Message::server("Set username succesfully")
                })
                .unwrap_or(common::Message::error("Username already exists"))
        })
        .unwrap_or(common::Message::error("Invalid Username"))
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}

fn user_name_exists<T: Hash>(map: &HashMap<T, User>, username: &str) -> bool {
    map.values()
        .find(|user| user.user_data.read().unwrap().user_name == username)
        .is_some()
}
