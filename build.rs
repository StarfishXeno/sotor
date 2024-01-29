use internal::{util::Game, GameData};
use std::{
    env::var,
    path::{Path, PathBuf},
};

fn make_game_dir(common: &Path, name: &str) -> PathBuf {
    let mut dir = common.to_owned();
    dir.push(name);
    assert!(dir.exists(), "game directory missing at {dir:#?}");
    dir
}

fn main() {
    dotenv::dotenv().ok();
    let steam_dir: PathBuf = var("STEAM_APPS").unwrap().into();
    let mut common = steam_dir.clone();
    common.push("common");
    let game_dirs = Game::LIST.map(|game| make_game_dir(&common, game.steam_dir()));
    let game_data = Game::LIST
        .map(|game| GameData::read(game, &game_dirs[game.idx()], Some(&steam_dir)).unwrap());
    let out_dir = var("OUT_DIR").unwrap();
    let out = &mut std::fs::File::options()
        .write(true)
        .create(true)
        .open(PathBuf::from_iter([&out_dir, "gamedata.zip"]))
        .unwrap();
    let mut zip = zip::ZipWriter::new(out);

    let method = if var("TARGET").unwrap() == "wasm32-unknown-unknown" {
        zip::CompressionMethod::Deflated
    } else {
        zip::CompressionMethod::Bzip2
    };
    let options = zip::write::FileOptions::default().compression_method(method);

    zip.start_file("gamedata.bin", options).unwrap();

    bincode::serialize_into(&mut zip, &game_data).unwrap();
    zip.finish().unwrap();
}
