use core::{util::Game, GameData};
use std::{env::var, fs, io::Write, path::PathBuf};

fn main() {
    dotenv::dotenv().ok();
    let out_dir = var("OUT_DIR").unwrap();
    let out = &mut fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PathBuf::from_iter([&out_dir, "assets.zip"]))
        .unwrap();
    let mut zip = zip::ZipWriter::new(out);
    let method = if var("TARGET").unwrap() == "wasm32-unknown-unknown" {
        zip::CompressionMethod::Deflated
    } else {
        zip::CompressionMethod::Bzip2
    };
    let options = zip::write::FileOptions::default().compression_method(method);

    zip.start_file("gamedata.bin", options).unwrap();
    let steam_dir: PathBuf = var("STEAM_APPS").unwrap().into();
    let mut common = steam_dir.clone();
    common.push("common");
    let game_dirs = Game::LIST.map(|game| {
        let mut dir = common.clone();
        dir.push(game.steam_dir());
        assert!(dir.exists(), "game directory missing at {dir:#?}");
        dir
    });
    let game_data = Game::LIST
        .map(|game| GameData::read(game, &game_dirs[game.idx()], Some(&steam_dir)).unwrap());
    bincode::serialize_into(&mut zip, &game_data).unwrap();

    zip.start_file("icons.ttf", options).unwrap();
    let icons = fs::read(PathBuf::from_iter(["assets", "fa-solid-900.ttf"])).unwrap();
    zip.write_all(&icons).unwrap();

    zip.start_file("font.ttf", options).unwrap();
    let font = fs::read(PathBuf::from_iter(["assets", "Roboto-Medium.ttf"])).unwrap();
    zip.write_all(&font).unwrap();

    zip.finish().unwrap();
}
