mod chats;
mod chats_map;
mod connection;

use async_std::prelude::*;
use async_std::{net, task};
use std::sync::Arc;

use crate::connection::handle;
use chat::utils::ChatResult;

fn main() -> ChatResult<()> {
    let addr = std::env::args().nth(1).expect("server ADDRESS");

    let chat_table = Arc::new(chats_map::ChatTracker::new());

    async_std::task::block_on(async {
        // Set up socket
        let listener = net::TcpListener::bind(addr).await?;

        // Handle incoming connections
        let mut new_connections = listener.incoming();

        while let Some(socket_result) = new_connections.next().await {
            let socket = socket_result?;
            let chats = chat_table.clone();

            task::spawn(async { log_error(handle(socket, chats).await) });
        }
        Ok(())
    })
}

fn log_error(result: ChatResult<()>) {
    if let Err(error) = result {
        println!("Error: {}", error);
    }
}
