#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod device;
mod error;

fn main() {
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
