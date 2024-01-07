use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub mod utils;

// Enum that defines messages that can be sent to the server [[PartialEq to allow for == & !=]]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Client {
    Join {
        chat_name: Arc<String>, //Thread safe pointer to a string
    },
    Post {
        chat_name: Arc<String>,
        message: Arc<String>,
    },
}

// Enum that defines messages that can be sent to the client
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Server {
    Message {
        chat_name: Arc<String>,
        message: Arc<String>,
    },
    Error(String),
}
