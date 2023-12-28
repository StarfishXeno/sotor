use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read as _, Write as _},
    path::PathBuf,
};

// read the whole file into a buffer
pub fn read_file(path: PathBuf) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::with_capacity(file.metadata()?.len() as usize);
    file.read_to_end(&mut buf)?;

    Ok(buf)
}

// write the whole buffer into a file
pub fn write_file(path: &str, buf: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(buf)?;

    Ok(())
}

// map of lowercase -> real filenames in a dir
pub fn read_dir_filemap(path: &str) -> io::Result<HashMap<String, String>> {
    let dir = fs::read_dir(path)?;
    let names: Vec<_> = dir
        .map(|e| e.unwrap().file_name().to_str().unwrap().to_owned())
        .collect();
    let map = names.into_iter().map(|s| (s.to_lowercase(), s)).collect();

    Ok(map)
}

pub struct Directory {
    pub path: String,
    pub name: String,
}

// list of paths to dirs in a dir
pub fn read_dir_dirs(path: &str) -> io::Result<Vec<Directory>> {
    let dir = fs::read_dir(path)?;
    let paths = dir
        .filter_map(|d| {
            let entry = d.unwrap();
            if entry.metadata().unwrap().is_dir() {
                Some(Directory {
                    path: entry.path().to_str().unwrap().to_owned(),
                    name: entry.file_name().to_str().unwrap().to_owned(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(paths)
}

pub fn file_exists(path: PathBuf) -> bool {
    fs::File::open(path).is_ok()
}
