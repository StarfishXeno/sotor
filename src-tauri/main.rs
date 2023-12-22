// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use formats::{
    erf::{self, Erf},
    gff::{self, Gff},
};
use save::Save;

mod formats;
mod save;
mod util;

#[tauri::command]
fn read_from_firectory(path: &str) -> Result<Save, String> {
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
        .invoke_handler(tauri::generate_handler![read_from_firectory])
        .invoke_handler(tauri::generate_handler![write_gff])
        .invoke_handler(tauri::generate_handler![write_erf])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
