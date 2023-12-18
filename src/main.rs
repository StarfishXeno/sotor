use crate::{
    formats::{erf, gff},
    util::read_file,
};

mod formats;
mod util;

const NFO_PATH: &str = "./save/savenfo.res";
const ERF_PATH: &str = "./save/savegame.sav";
fn main() {
    use std::time::Instant;
    let now = Instant::now();

    let nfo_bytes = read_file(NFO_PATH).unwrap();
    let nfo = gff::read(&nfo_bytes);
    if let Ok(gff) = nfo {
        println!("{:?}", gff.content.fields.keys());
        let bytes = gff::write(gff.clone()).unwrap();
        let new_gff = gff::read(&bytes).unwrap();
        debug_assert_eq!(gff, new_gff);
    } else {
        println!("{nfo:?}");
    }
    println!("//////////////////");
    let erf_bytes = read_file(ERF_PATH).unwrap();
    let erf = erf::read(&erf_bytes);
    println!("{erf:#?}");
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
