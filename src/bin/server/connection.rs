// Represents single connection between server and client

use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};

use crate::chats_map::ChatTracker;
use chat::utils::{self, ChatResult};
use chat::{Client, Server};

pub struct Leaving(Mutex<TcpStream>);
impl Leaving {
    pub fn new(client: TcpStream) -> Leaving {
        Leaving(Mutex::new(client))
    }

    pub async fn send(&self, packet: Server) -> ChatResult<()> {
        let mut lock = self.0.lock().await;

        utils::send_json(&mut *lock, &packet).await?;

        lock.flush().await?;

        Ok(())
    }
}

pub async fn handle(socket: TcpStream, chats: Arc<ChatTracker>) -> ChatResult<()> {
    let leaving = Arc::new(Leaving::new(socket.clone()));

    let buffered = BufReader::new(socket);

    let mut from_client = utils::receive(buffered);

    // loop through messages from client in the buffer
    while let Some(req_res) = from_client.next().await {
        let request = req_res?;

        let result = match request {
            Client::Join { chat_name } => {
                let chat = chats.find_or_new(chat_name);
                chat.join(leaving.clone());
                Ok(())
            }
            Client::Post { chat_name, message } => match chats.find(&chat_name) {
                Some(chat) => {
                    chat.post(message);
                    Ok(())
                }
                None => Err(format!("Chat does not exist: {}", chat_name)),
            },
        };

        if let Err(message) = result {
            let report = Server::Error(message);
            leaving.send(report).await?
        }
    }

    Ok(())
}
