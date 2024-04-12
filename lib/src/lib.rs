//! # HTTP Requests Library
//!
//! This library provides a simple and easy-to-use interface for testing APIs and making HTTP requests.
//! and handling responses in Rust.
//!
//! ## Contributing
//!
//! Contributions are welcome! If you find any issues or have suggestions for improvements,
//! please open an issue or submit a pull request on the GitHub repository.
//!
//! ## License
//!
//! This library is licensed under the [AGPL-3.0](https://choosealicense.com/licenses/agpl-3.0/).

mod http;
mod websocket;

pub use http::{HttpRequest, HttpResponse, HttpRequestGroup, send_http_request};
pub use websocket::WebSocketManager;