use std::time;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub topic: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Acknowledgement {
    pub date: time::SystemTime,
    pub msg_id: String,
}
// temporarily containing string
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Message {
    /// Acknowledges a message
    Ack(Acknowledgement),
    Success,
    /// Post a message to a topic.
    /// Should be responded to with Success
    Post(Post),
    /// Subscribes the connected user to a topic
    /// Should be met with Success
    Subscribe(String),
    /// A message dispatched to subscribers
    /// Should be met with Ack
    Publish(Post),
    Err,
}

pub enum HandshakeInit {
    ConnectionDetails {
        username: String,
        password: String
    }
}

pub enum Handshake {
    Init(HandshakeInit),
    Accept
}

// To be renamed later
pub enum NewMessage {
    Handshake(Handshake),
    Subscribe,
    Publish,
}

#[cfg(test)]
mod tests {}
