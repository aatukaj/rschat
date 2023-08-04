use std::{
    collections::HashMap,
    env,
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
    user_data: RwLock<UserData>,
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
        user_id: 69,
        user_name: "SERVER".into(),
        content: format!("connected succesfully as '{}'", new_user_data.user_name).into(),
        color: ratatui::style::Color::Cyan,
    }
    .serialize();

    let (tx, rx) = unbounded();
    tx.unbounded_send(Message::Binary(msg)).unwrap();

    peer_map.lock().unwrap().insert(
        addr,
        User {
            user_data: RwLock::new(UserData {
                color: new_user_data.color,
                user_name: new_user_data.user_name.into(),
            }),
            tx,
        },
    );

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let msg = msg.to_text().unwrap();

        println!(
            "Received a message from {}: {}",
            addr,
            msg
        );
        let peers = peer_map.lock().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients = peers.values().map(|user| &user.tx);

        let user_data = peers.get(&addr).unwrap().user_data.read().unwrap();

        let msg_to_send = common::Message {
            user_id: 0,
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

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
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
