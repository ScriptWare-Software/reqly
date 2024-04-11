use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::sync::mpsc;
use futures::{future::FutureExt, stream::StreamExt, SinkExt};
use url::Url;
use std::error::Error;

pub struct WebSocketManager {
    sender: mpsc::Sender<WebSocketCommand>,
}

enum WebSocketCommand {
    SendMessage(usize, String),
    Close(usize),
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::channel(32);
        let manager = WebSocketManager { sender };

        tokio::spawn(async move {
            let mut connections = vec![];

            while let Some(command) = receiver.recv().await {
                match command {
                    WebSocketCommand::SendMessage(id, msg) => {
                        if let Some(conn) = connections.get_mut(id) {
                            let _ = conn.send(Message::Text(msg)).await;
                        }
                    }
                    WebSocketCommand::Close(id) => {
                        if id < connections.len() {
                            let _ = connections.remove(id);
                        }
                    }
                }
            }
        });

        manager
    }

    pub fn connect(&self, url: &str) -> Result<usize, Box<dyn Error>> {
        let url = Url::parse(url)?;
        let sender = self.sender.clone();

        tokio::spawn(async move {
            let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
            let (write, mut read) = ws_stream.split();

            let (tx, mut rx) = mpsc::channel(32);
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    write.send(msg).await.expect("Failed to send message");
                }
            });

            while let Some(message) = read.next().await {
                match message.expect("Failed to read message") {
                    Message::Text(txt) => println!("Received: {}", txt),
                    Message::Close(_) => break,
                    _ => (),
                }
            }

            let _ = sender.send(WebSocketCommand::Close(0)).await; // Replace with correct ID
        });

        // Return connection ID
        Ok(0) // Replace with correct ID management
    }

    pub fn send_message(&self, connection_id: usize, message: String) {
        let _ = self.sender.send(WebSocketCommand::SendMessage(connection_id, message));
    }

    pub fn close_connection(&self, connection_id: usize) {
        let _ = self.sender.send(WebSocketCommand::Close(connection_id));
    }
}