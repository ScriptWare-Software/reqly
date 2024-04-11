/// This file contains the core functionality for sending HTTP requests and handling responses.
///
/// It defines the following main types:
/// - `HttpRequest`: Represents an HTTP request with a URL, method, headers, and optional body.
/// - `HttpResponse`: Represents an HTTP response with a status code, headers, and body.
/// - `HttpRequestGroup`: Represents a group of HTTP requests.
///
/// The `send_http_request` function is the primary entry point for sending an HTTP request.
/// It takes an `HttpRequest` as input and returns an `HttpResponse` wrapped in a `Result`.
///
/// The file also includes test cases to verify the functionality of sending GET, POST, and
/// invalid HTTP requests using the `send_http_request` function.
///
/// Dependencies:
/// - `curl::easy::{Easy, List}`: Used for making HTTP requests and handling low-level details.
/// - `serde::{Deserialize, Serialize}`: Used for serializing and deserializing structs.
/// - `std::str`: Used for string manipulation and conversion.

use curl::easy::{Easy, List};
use serde::{Deserialize, Serialize};
use std::str;

/// Represents an HTTP request.
#[derive(Serialize, Deserialize)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
}

/// Represents an HTTP response.
#[derive(Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u32,
    pub headers: Vec<String>,
    pub body: String,
}

/// Represents a group of HTTP requests.
#[derive(Serialize, Deserialize)]
pub struct HttpRequestGroup {
    pub requests: Vec<HttpRequest>,
}

/// Sends an HTTP request and returns the corresponding response.
///
/// # Arguments
///
/// * `request` - The HTTP request to send.
///
/// # Returns
///
/// A `Result` containing the HTTP response if the request was successful,
/// or a `String` error message if the request failed.
pub fn send_http_request(request: HttpRequest) -> Result<HttpResponse, String> {
    let mut easy = Easy::new();
    easy.url(&request.url).map_err(|e| e.to_string())?;

    match request.method.as_str() {
        "GET" => easy.get(true),
        "POST" => easy.post(true),
        "PUT" => easy.put(true),
        "DELETE" => easy.custom_request("DELETE"),
        "HEAD" => easy.nobody(true),
        "OPTIONS" => easy.custom_request("OPTIONS"),
        "TRACE" => easy.custom_request("TRACE"),
        "PATCH" => easy.custom_request("PATCH"),
        "CONNECT" => easy.custom_request("CONNECT"),
        _ => easy.custom_request(&request.method),  // handle any other custom methods
    }
    .map_err(|e| e.to_string())?;

    let mut headers_list = List::new();
    for header in request.headers {
        headers_list.append(&header).map_err(|e| e.to_string())?;
    }
    easy.http_headers(headers_list).map_err(|e| e.to_string())?;

    if let Some(body) = request.body {
        easy.post_fields_copy(body.as_bytes()).map_err(|e| e.to_string())?;
    }

    let mut response_body = Vec::new();
    let mut header_buffer = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                response_body.extend_from_slice(data);
                Ok(data.len())
            })
            .map_err(|e| e.to_string())?;
        transfer
            .header_function(|header_data| {
                header_buffer.extend_from_slice(header_data);
                true
            })
            .map_err(|e| e.to_string())?;
        transfer.perform().map_err(|e| e.to_string())?;
    }

    let status_code = easy.response_code().map_err(|e| e.to_string())?;
    let headers = str::from_utf8(&header_buffer)
        .map_err(|e| e.to_string())?
        .split("\r\n")
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    Ok(HttpResponse {
        status: status_code,
        headers,
        body: String::from_utf8(response_body).map_err(|e| e.to_string())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests sending a GET request.
    #[test]
    fn test_send_http_request_get() {
        let request = HttpRequest {
            url: "https://reqres.in/api/users?page=2".to_string(),
            method: "GET".to_string(),
            headers: vec![],  // No predefined headers for this test
            body: None,
        };

        let response = send_http_request(request).unwrap();
        assert_eq!(response.status, 200);
        assert!(!response.body.is_empty());
    }

    /// Tests sending a POST request.
    #[test]
    fn test_send_http_request_post() {
        let request = HttpRequest {
            url: "https://reqres.in/api/register".to_string(),
            method: "POST".to_string(),
            headers: vec!["Content-Type: application/json".to_string()],
            body: Some(r#"{"email": "eve.holt@reqres.in", "password": "pistol"}"#.to_string()),
        };

        let response = send_http_request(request).unwrap();
        assert_eq!(response.status, 200);
        assert!(!response.body.is_empty());
    }

    /// Tests sending a request with an invalid HTTP method.
    #[test]
    fn test_send_http_request_invalid_method() {
        let request = HttpRequest {
            url: "https://api.example.com/endpoint".to_string(),
            method: "INVALID".to_string(),
            headers: vec![],
            body: None,
        };

        let response = send_http_request(request);
        assert!(response.is_err());
    }
}
