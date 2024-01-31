use crate::util::Game;
pub use core::util::fs::*;
use std::{fs, io, path::PathBuf, process::Command};

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

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
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

    if cfg!(target_os = "windows") {
        let Ok(app_data) = std::env::var("LOCALAPPDATA") else {
            return paths;
        };
        let game_dir = match game {
            Game::One => "SWKotOR",
            Game::Two => "SWKotORII",
        };

        let path_end = PathBuf::from_iter(["LucasArts", game_dir]);
        let mut path = PathBuf::from_iter(["VirtualStore", &app_data]);
        let mut path_x86 = path.clone();
        path.push("Program Files");
        path_x86.push("Program Files (x86)");
        path.push(path_end.clone());
        path_x86.push(path_end);

        paths.push(path);
        paths.push(path_x86);
    } else if cfg!(target_os = "linux") {
        if game == Game::One {
            return paths;
        }
        let Ok(home) = std::env::var("HOME") else {
            return paths;
        };
        let path = PathBuf::from_iter([&home, ".local", "share", "aspyr-media", "kotor2"]);

        paths.push(path);
    }

    paths
}

pub fn select_directory(title: String) -> Option<String> {
    rfd::FileDialog::new()
        .set_title(title)
        .pick_folder()
        .map(|pb| pb.to_str().unwrap().to_owned())
}
