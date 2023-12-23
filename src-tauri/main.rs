// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use formats::{
    erf::{self, Erf},
    gff::{self, Gff},
};
use save::Save;
use tauri::Manager;

mod formats;
mod save;
mod util;

#[tauri::command]
fn read_from_directory(path: &str) -> Result<Save, String> {
    Save::read_from_directory(path)
}

#[tauri::command]
fn write_gff(gff: Gff) -> Vec<u8> {
    gff::write(gff)
}

#[tauri::command]
fn write_erf(erf: Erf) -> Vec<u8> {
    erf::write(erf)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            read_from_directory,
            write_gff,
            write_erf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
