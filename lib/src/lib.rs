use curl::easy::{Easy, List};
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Serialize, Deserialize)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u32,
    pub headers: Vec<String>,
    pub body: String,
}

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

    #[test]
    fn test_send_http_request_post() {
        let request = HttpRequest {
            url: "https://reqres.in/api/register".to_string(),
            method: "POST".to_string(),
            headers: vec!["Content-Type: application/json".to_string()],
            body: Some(r#"{"email": "eve.holt@reqres.in", "password": "pistol"}"#.to_string()),
        };

        let response = send_http_request(request).unwrap();
        assert_eq!(response.status, 200); // Make sure this status code aligns with your API's behavior
        assert!(!response.body.is_empty());
    }

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
