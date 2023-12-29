use rfd::{AsyncFileDialog, FileHandle};
use std::{collections::HashMap, fs, future::Future, io, path::PathBuf, time::SystemTime};

pub fn file_exists(path: PathBuf) -> bool {
    fs::File::open(path).is_ok()
}

// map of lowercase -> real filenames in a dir
pub fn read_dir_filemap(path: PathBuf) -> io::Result<HashMap<String, String>> {
    let dir = fs::read_dir(path)?;
    let names: io::Result<Vec<_>> = dir
        .map(|e| Ok(e?.file_name().to_str().unwrap().to_owned()))
        .collect();
    let map = names?.into_iter().map(|s| (s.to_lowercase(), s)).collect();

    Ok(map)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directory {
    pub path: String,
    pub name: String,
    pub date: u64,
}

// list of paths to dirs in a dir
pub fn read_dir_dirs(path: PathBuf) -> io::Result<Vec<Directory>> {
    let dir = fs::read_dir(path)?;
    let paths: Vec<_> = dir
        .filter_map(|entry| {
            let Ok(entry) = entry else {
                return None;
            };
            let Ok(metadata) = entry.metadata() else {
                return None;
            };
            if !metadata.is_dir() {
                return None;
            }

            let dir = Directory {
                path: entry.path().to_str()?.to_owned(),
                name: entry.file_name().to_str()?.to_owned(),
                date: metadata
                    .created()
                    .ok()?
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .ok()?
                    .as_secs(),
            };

            Some(dir)
        })
        .collect();

    Ok(paths)
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
