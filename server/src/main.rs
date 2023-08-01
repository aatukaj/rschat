use common::Message;
use serde;
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
}
impl Client {
    fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            id: self.id,
            user_name: Arc::clone(&self.user_name),
            stream: self.stream.try_clone()?,
        })
    }
}

const USER_NAMES: &[&str] = &["bob", "patrick", "nobert"];


fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::default().init()?;
    let listener = TcpListener::bind("127.0.0.1:80")?;
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = Vec::new();
    for (i, stream) in listener.incoming().enumerate() {
        let mut stream = stream?;
        log::info!("New Connection {:?}", stream);
        let msg =Message {
            user: 69,
            content: "connected to server succesfully!\n".into(),
        };
        stream.write(&msg.serialize())?;
        let id = i as u32;

        let client = Client {
            stream,
            id,
            user_name: format!("{}{}", USER_NAMES[i % USER_NAMES.len()], i).into()
        };

        let mut lock = clients.lock().unwrap();
        
        lock.insert(
            id,
            client.try_clone()?       
        );
        let clients = Arc::clone(&clients);

        handles.push(thread::spawn(move || loop {
            let mut buf = Vec::new();

            if let Err(err) = BufReader::new(&client.stream).read_until(b'\n', &mut buf) {
                log::warn!("{}", err);
                clients.lock().unwrap().remove(&id);
                break;
            }

            println!("{:?}", str::from_utf8(&buf).unwrap());
            let bytes_to_send = &common::Message {
                user: client.id,
                content: str::from_utf8(&buf).unwrap().into(),
            }.serialize();

            let mut lock = clients.lock().unwrap();
            for (_, client) in lock.iter_mut() {
                client.stream.write_all(bytes_to_send).unwrap_or_else(|err| log::error!("{}", err));
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}
