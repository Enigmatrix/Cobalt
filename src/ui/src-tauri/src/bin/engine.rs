//! Entry point for the engine, exposed to tauri.
//! This is done so that the engine is auto-bundle has its file properties set.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    engine::main();
}
