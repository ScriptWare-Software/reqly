use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tokio::sync::{mpsc, Mutex};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use url::Url;
use std::error::Error;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

type WebSocketConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct WebSocketManager {
    sender: mpsc::Sender<WebSocketCommand>,
    connections: Arc<Mutex<Vec<WebSocketConnection>>>, // shared state across tasks
}

enum WebSocketCommand {
    SendMessage(usize, String),
    Close(usize),
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::channel(32);
        let connections = Arc::new(Mutex::new(Vec::new())); // Initialize with a shared, mutable vector
        let manager = WebSocketManager { sender, connections: connections.clone() };

        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    WebSocketCommand::SendMessage(id, msg) => {
                        if let Some(conn) = connections.lock().await.get_mut(id) {
                            let _ = conn.send(Message::Text(msg)).await;
                        }
                    }
                    WebSocketCommand::Close(id) => {
                        let mut conns = connections.lock().await;
                        if id < conns.len() {
                            conns.remove(id); // Properly remove the connection
                        }
                    }
                }
            }
        });

        manager
    }

    pub async fn connect(&self, url: &str) -> Result<usize, Box<dyn Error>> {
        let url = Url::parse(url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let mut conns = self.connections.lock().await;

        let id = conns.len();  // Get new ID for the connection
        conns.push(ws_stream); // Store the connection

        Ok(id) // Return the new connection ID
    }

    pub async fn send_message(&self, connection_id: usize, message: String) -> Result<(), Box<dyn Error>> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(connection_id) {
            conn.send(Message::Text(message)).await?;
        }
        Ok(())
    }

    pub async fn close_connection(&self, connection_id: usize) {
        let _ = self.sender.send(WebSocketCommand::Close(connection_id)).await;
    }
}


#[tokio::test]
async fn test_websocket_manager() {
    let manager = WebSocketManager::new();

    let connection_id = manager.connect("").await.unwrap();

    let message = "hello!".to_string();
    manager.send_message(connection_id, message.clone()).await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let mut conns = manager.connections.lock().await;
    let conn = conns.get_mut(connection_id).unwrap();
    let msg = conn.next().await.unwrap().unwrap();
    
    assert_eq!(msg, Message::Text(message));

    manager.close_connection(connection_id).await;
    assert_eq!(conns.len(), 0);
}