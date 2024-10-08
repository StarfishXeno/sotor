use zip::ZipWriter;

use crate::util::Game;
pub use core::util::fs::*;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

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

pub fn add_zip_file(path: &PathBuf, zip: &mut ZipWriter<&mut File>) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let options = zip::write::FileOptions::default();

    let file = fs::read(path)?;
    zip.start_file(path.file_name().unwrap().to_str().unwrap(), options)?;
    zip.write_all(&file)?;

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
        let path = PathBuf::from_iter([&app_data, "VirtualStore"]);
        for pf in ["Program Files", "Program Files (x86)"] {
            paths.push(PathBuf::from_iter([&path, &PathBuf::from(pf), &path_end]));
        }
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
