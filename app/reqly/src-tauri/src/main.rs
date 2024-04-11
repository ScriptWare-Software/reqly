// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqly::{HttpRequest, HttpResponse, send_http_request};
use tauri::command;

#[command]
fn perform_http_request(request: HttpRequest) -> Result<HttpResponse, String> {
    send_http_request(request)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, perform_http_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}