use common::Message;
use ratatui::style::Color;
use std::collections::HashMap;
use std::io::{self, prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;

struct Client {
    id: u32,
    user_name: Arc<str>,
    stream: TcpStream,
    color: Color,
}
impl Client {
    fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            id: self.id,
            user_name: Arc::clone(&self.user_name),
            stream: self.stream.try_clone()?,
            color: self.color,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::default().init()?;
    let listener = TcpListener::bind("127.0.0.1:80")?;
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = Vec::new();
    for (i, stream) in listener.incoming().enumerate() {
        let id = i as u32;
        let stream = stream?;
        log::info!("New Connection {:?}", stream);

        let clients = Arc::clone(&clients);

        handles.push(thread::spawn(move || handle_client(stream, id, clients)));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}

fn handle_client(stream: TcpStream, id: u32, clients: Arc<Mutex<HashMap<u32, Client>>>) {
    let mut buf = Vec::new();
    BufReader::new(&stream).read_until(b'\n', &mut buf).unwrap();
    let user_data: common::NewUserSet = serde_json::from_slice(&buf).unwrap();

    let mut client = Client {
        stream,
        id,
        user_name: user_data.user_name.into(),
        color: user_data.color,
    };

    clients
        .lock()
        .unwrap()
        .insert(id, client.try_clone().unwrap());

    let msg = Message {
        user_id: 69,
        user_name: "SERVER".into(),
        content: format!("connected succesfully as '{}'", client.user_name).into(),
        color: ratatui::style::Color::Cyan,
    };

    client.stream.write_all(&msg.serialize()).unwrap();

    loop {
        let mut buf = Vec::new();

        if let Err(err) = BufReader::new(&client.stream).read_until(b'\n', &mut buf) {
            log::warn!("{}", err);
            clients.lock().unwrap().remove(&client.id);
            break;
        }

        println!("{:?}", str::from_utf8(&buf).unwrap());

        let bytes_to_send = &common::Message {
            user_id: client.id,
            user_name: client.user_name.as_ref().into(),
            content: str::from_utf8(&buf).unwrap().into(),
            color: client.color,
        }
        .serialize();

        let mut lock = clients.lock().unwrap();
        for (_, client) in lock.iter_mut() {
            client
                .stream
                .write_all(bytes_to_send)
                .unwrap_or_else(|err| log::error!("{}", err));
        }
    }
}
