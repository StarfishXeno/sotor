use crate::util::Game;
pub use internal::util::fs::*;
use rfd::{AsyncFileDialog, FileHandle};
#[cfg(not(target_os = "windows"))]
use std::os::unix::process::CommandExt;
use std::{fs, future::Future, io, path::PathBuf, process::Command};

pub fn open_file_manager(path: &str) {
    let program = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        return;
    };

    let mut command = Command::new(program);

    #[cfg(not(target_os = "windows"))]
    {
        command.process_group(0);
    }

    command.arg(path).spawn().unwrap();
}

pub fn backup_file(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut target_path = path.clone();
    target_path.set_extension(
        target_path
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_string()
            + ".bak",
    );
    fs::copy(path, target_path)?;

    Ok(())
}

pub fn get_extra_save_directories(game: Game) -> Vec<PathBuf> {
    let mut paths = vec![];

    if game == Game::One {
        return paths;
    }

    if cfg!(target_os = "windows") {
        let Ok(app_data) = std::env::var("LocalAppData") else {
            return paths;
        };

        let path_end = PathBuf::from_iter(["LucasArts", "SWKotORII"]);
        let mut path = PathBuf::from(app_data);
        let mut path_x86 = path.clone();
        path.push("Program Files");
        path_x86.push("Program Files (x86)");
        path.push(path_end.clone());
        path_x86.push(path_end);

        paths.push(path);
        paths.push(path_x86);
    } else if cfg!(target_os = "linux") {
        let Ok(home) = std::env::var("HOME") else {
            return paths;
        };
        let path = PathBuf::from_iter([&home, ".local", "share", "aspyr-media", "kotor2"]);

        paths.push(path);
    }

    paths
}

pub fn select_directory(title: String) -> Option<FileHandle> {
    execute(async move { AsyncFileDialog::new().set_title(title).pick_folder().await })
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<T: 'static + Send, F: Future<Output = T> + Send + 'static>(f: F) -> T {
    futures_lite::future::block_on(f)
}
#[cfg(target_arch = "wasm32")]
fn execute<T: 'static + Send, F: Future<Output = T> + Send + 'static>(f: F) -> T {
    wasm_bindgen_futures::spawn_local(f);
}
