use async_std::prelude::*;
use async_std::{io, net, task};
use futures_lite::future::FutureExt;
use std::sync::Arc;

use chat::utils::{self, ChatResult};
use chat::{Client, Server};

// helper function to check the arguments and either join / post  or error
fn parse_input(line: &str) -> Option<Client> {
    let (input, remainder) = get_valve(line)?;

    if input == "join" {
        let (chat, remainder) = get_valve(remainder)?;
        if !remainder.trim_start().is_empty() {
            return None;
        }
        return Some(Client::Join {
            chat_name: Arc::new(chat.to_string()),
        });
    } else if input == "post" {
        let (chat, remainder) = get_valve(remainder)?;
        let message = remainder.trim_start().to_string();

        return Some(Client::Post {
            chat_name: Arc::new(chat.to_string()),
            message: Arc::new(message),
        });
    } else {
        println!("Unrecognized input: {:?}", line);
        return None;
    }
}

// helper function if input is none return none, if input has no white space return (input, ), if input has whitespace return (arg1, arg2)
fn get_valve(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();

    if input.is_empty() {
        return None;
    }

    match input.find(char::is_whitespace) {
        Some(whitespace) => Some((&input[0..whitespace], &input[whitespace..])),
        None => Some((input, "")),
    }
}

// Send to server
async fn send(mut send: net::TcpStream) -> ChatResult<()> {
    println!("Options \n join CHAT \n post CHAT MESSAGE");

    // Creates a new buffer stream from the command line
    let mut options = io::BufReader::new(io::stdin()).lines();

    while let Some(option_result) = options.next().await {
        let opt = option_result?;
        let req = match parse_input(&opt) {
            Some(req) => req,
            None => continue,
        };
        utils::send_json(&mut send, &req).await?;
        send.flush().await?;
    }
    Ok(())
}

// Receive messages from server
async fn messages(server: net::TcpStream) -> ChatResult<()> {
    // Creates a new buffer stream from the command line
    let buf = io::BufReader::new(server);

    let mut stream = utils::receive(buf);

    while let Some(msg) = stream.next().await {
        match msg? {
            Server::Message { chat_name, message } => {
                println!("Chat Name: {}\n, Message: {}\n", chat_name, message);
            }
            Server::Error(message) => {
                println!("Error received: {}\n", message);
            }
        }
    }
    Ok(())
}

fn main() -> ChatResult<()> {
    // read in address from command line arguments
    let addr = std::env::args().nth(1).expect("Address:PORT");

    task::block_on(async {
        // connects to the server using address from env args
        let socket = net::TcpStream::connect(addr).await?;

        socket.set_nodelay(true)?; // Disables features that reduces latency

        // Creates a new task for sending to the server
        let send = send(socket.clone());

        // Creates a new task for receiving from the server
        let replies = messages(socket);

        // races the send & replies in parallel and awaits on who comes back first
        replies.race(send).await?;

        Ok(())
    })
}
