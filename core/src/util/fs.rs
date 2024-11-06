use ahash::HashMap;
use std::{
    fs,
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
    time::SystemTime,
};

pub fn read_file(dir: impl AsRef<Path>, file: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut path = dir.as_ref().to_path_buf();
    for part in file.as_ref() {
        let map = read_dir_filemap(&path)?;
        let real_name = map
            .get(&part.to_string_lossy().to_lowercase())
            .ok_or_else(|| {
                Error::new(ErrorKind::NotFound, format!("couldn't find file: {part:?}"))
            })?;
        path.push(real_name);
    }

    fs::read(path)
}

// map of lowercase -> real filenames in a dir
pub fn read_dir_filemap(path: &PathBuf) -> io::Result<HashMap<String, String>> {
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
                    .modified()
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
