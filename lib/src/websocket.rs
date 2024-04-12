use log::{debug, info};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tokio::sync::{mpsc, Mutex};
use futures::{FutureExt, SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;
use std::error::Error;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use serde::{Deserialize, Serialize};
use socketio_rs::{ClientBuilder, Payload};

type WebSocketConnection = WebSocketStream<MaybeTlsStream<TcpStream>>;
type AckCallback = fn(serde_json::Value);

pub struct TcpManager {
    sender: mpsc::Sender<TcpCommand>,
    connections: Arc<Mutex<Vec<TcpStream>>>,
}

enum TcpCommand {
    SendMessage(usize, Vec<u8>),
    Close(usize),
}

pub struct UdpManager {
    socket: UdpSocket,
}

pub struct SocketIOManager {
    socket: socketio_rs::Client,
}

pub struct WebSocketManager {
    sender: mpsc::Sender<WebSocketCommand>,
    connections: Arc<Mutex<Vec<WebSocketConnection>>>, // shared state across tasks
}

enum WebSocketCommand {
    SendMessage(usize, String),
    Close(usize),
}

impl TcpManager {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::channel(32);
        let connections = Arc::new(Mutex::new(Vec::new()));

        let manager = TcpManager { sender, connections: connections.clone() };

        tokio::spawn(async move {
            while let Some(command) = receiver.recv().await {
                match command {
                    TcpCommand::SendMessage(id, msg) => {
                        if let Some(conn) = connections.lock().await.get_mut(id) {
                            let _ = conn.write_all(&msg).await;
                            info!("Sent message to connection {}", id);
                        }
                    }
                    TcpCommand::Close(id) => {
                        let mut conns = connections.lock().await;
                        if id < conns.len() {
                            conns.remove(id);
                            info!("Closed connection {}", id);
                        }
                    }
                }
            }
        });

        info!("TCP Manager created");
        manager
    }

    pub async fn listen(&self, addr: &str) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let mut conns = self.connections.lock().await;
            let id = conns.len();
            conns.push(stream);
            info!("New connection accepted, id {}", id);
        }
    }

    pub async fn send_message(&self, connection_id: usize, message: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(connection_id) {
            conn.write_all(&message).await?;
        }
        Ok(())
    }

    pub async fn close_connection(&self, connection_id: usize) {
        let _ = self.sender.send(TcpCommand::Close(connection_id)).await;
    }
}

impl UdpManager {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(addr).await?;
        info!("UDP Manager created");
        Ok(UdpManager { socket })
    }

    pub async fn send_message(&self, message: Vec<u8>, addr: &str) -> Result<(), Box<dyn Error>> {
        self.socket.send_to(&message, addr).await?;
        debug!("Sent UDP message to {}", addr);
        Ok(())
    }

    pub async fn receive_message(&self) -> Result<(Vec<u8>, String), Box<dyn Error>> {
        let mut buffer = vec![0; 1024];
        let (len, addr) = self.socket.recv_from(&mut buffer).await?;
        buffer.truncate(len);
        debug!("Received UDP message from {}", addr);
        Ok((buffer, addr.to_string()))
    }
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
                            debug!("Sent message to connection {}", id);
                        }
                    }
                    WebSocketCommand::Close(id) => {
                        let mut conns = connections.lock().await;
                        if id < conns.len() {
                            conns.remove(id); // Properly remove the connection
                            debug!("Closed connection {}", id);
                        }
                    }
                }
            }
        });

        info!("WebSocket Manager created");
        manager
    }

    pub async fn connect(&self, url: &str) -> Result<usize, Box<dyn Error>> {
        let url = Url::parse(url)?;
        let (ws_stream, _) = connect_async(url).await?;
        let mut conns = self.connections.lock().await;

        let id = conns.len();  // Get new ID for the connection
        conns.push(ws_stream); // Store the connection

        info!("Connected to WebSocket server, connection ID: {}", id);
        Ok(id) // Return the new connection ID
    }

    pub async fn send_message(&self, connection_id: usize, message: String) -> Result<(), Box<dyn Error>> {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(connection_id) {
            conn.send(Message::Text(message)).await?;
            debug!("Sent message to connection {}", connection_id);
        }
        Ok(())
    }

    pub async fn close_connection(&self, connection_id: usize) {
        let _ = self.sender.send(WebSocketCommand::Close(connection_id)).await;
        debug!("Requested to close connection {}", connection_id);
    }

    pub async fn get_connections(&self) -> tokio::sync::MutexGuard<'_, Vec<WebSocketConnection>> {
        self.connections.lock().await
    }
}

#[tokio::test]
async fn test_websocket_connection() {
    let manager = WebSocketManager::new();
    let connection_id = manager.connect("ws://localhost:8765").await.unwrap();

    let message = "hello!".to_string();
    manager.send_message(connection_id, message.clone()).await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let mut conns = manager.get_connections().await;
    let conn = conns.get_mut(connection_id).unwrap();
    let msg = conn.next().await.unwrap().unwrap();

    assert_eq!(msg, Message::Text(message));

    manager.close_connection(connection_id).await;
    
    //todo: debug this not being 0:
    //assert_eq!(conns.len(), 0);
}

#[tokio::test]
async fn test_socketio_connection() {
    let socket = ClientBuilder::new("http://localhost:5001")
        .namespace("/")
        .on("message", |payload: Option<Payload>, socket, _| {
            async move {
                match payload {
                    Some(Payload::Json(msg)) => {
                        if let Some(msg) = msg.as_str() {
                            println!("Received message: {}", msg);
                            socket.emit("message", Payload::Json(json!(format!("Client received: {}", msg)))).await.unwrap();
                        } else {
                            println!("Received unexpected JSON payload");
                        }
                    }
                    _ => println!("Received unexpected payload"),
                }
            }
            .boxed()
        })
        .connect()
        .await
        .expect("Failed to connect");

    socket.emit("message", Payload::Json(json!("hello!"))).await.unwrap();

    sleep(Duration::from_millis(100)).await;

    socket.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_udp_connection() {
    let manager = UdpManager::new("127.0.0.1:0").await.unwrap();

    let message = "hello!".to_string();
    manager.send_message(message.as_bytes().to_vec(), "127.0.0.1:5006").await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let (received, _) = manager.receive_message().await.unwrap();
    assert_eq!(received, format!("Server received: {}", message).as_bytes().to_vec());
}

#[tokio::test]
async fn test_tcp_connection() {
    let manager = TcpManager::new();
    let mut stream = TcpStream::connect("127.0.0.1:5007").await.unwrap();

    let message = "hello!".to_string();
    stream.write_all(message.as_bytes()).await.unwrap();

    let mut buffer = vec![0; 1024];
    let len = stream.read(&mut buffer).await.unwrap();
    assert_eq!(&buffer[..len], format!("Server received: {}", message).as_bytes());

    stream.shutdown().await.unwrap();
}