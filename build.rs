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
    dotenv::dotenv().unwrap();
    let mut steam_dir: PathBuf = var("STEAM_LIBRARY").unwrap().into();
    steam_dir.push("steamapps");
    let mut common = steam_dir.clone();
    common.push("common");
    let game_dirs = [
        make_game_dir(&common, "swkotor"),
        make_game_dir(&common, "Knights of the Old Republic II"),
    ];

    let mut game_data = vec![];
    for game in Game::LIST {
        game_data.push(GameData::read(game, &game_dirs[game.idx()], Some(&steam_dir)).unwrap());
    }
    let game_data: [GameData; Game::COUNT] = game_data.try_into().unwrap();
    let out_dir = var("OUT_DIR").unwrap();
    let out = &mut std::fs::File::options()
        .write(true)
        .create(true)
        .open(PathBuf::from_iter([&out_dir, "gamedata.bin"]))
        .unwrap();
    bincode::serialize_into(out, &game_data).unwrap();
}
