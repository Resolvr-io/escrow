// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use keyring::{Entry, Result};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn save_secret_key_to_keychain(nsec: &str, npub: &str) -> String {
    let entry = match Entry::new("resolvr", npub) {
        Ok(entry) => entry,
        Err(_e) => return format!("error"),
    };

    if let Err(_e) = entry.set_password(nsec) {
        return format!("error");
    }

    match entry.get_password() {
        // Ok(password) => format!("My password is '{}'", password),
        Ok(_password) => format!("success"),
        Err(_e) => format!("error"),
    }
}

#[tauri::command]
fn get_nsec(npub: &str) -> String {
    let entry = match Entry::new("resolvr", npub) {
        Ok(entry) => entry,
        Err(_e) => return format!("error"),
    };

    match entry.get_password() {
        // Ok(password) => format!("My password is '{}'", password),
        Ok(password) => format!("'{}'", password),
        Err(_e) => format!("error"),
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![save_secret_key_to_keychain])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
