// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Tauri main module

/// Auto-generated by Tauri
fn main() {
    app_lib::run();
}
