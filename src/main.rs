use crate::{
    formats::{erf, gff},
    util::read_file,
};

mod formats;
mod util;

const ERF_PATH: &str = "./save/savegame.sav";
fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let erf_bytes = read_file(ERF_PATH).unwrap();
    let erf = erf::read(&erf_bytes).unwrap();
    println!("{erf:#?}");
    let res = erf.resources.get("pc").unwrap();
    let gff = gff::read(&res.content).unwrap();
    println!("{:#?}", gff.content.fields.keys());
    let buf = gff::write(gff.clone()).unwrap();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    
    assert!(gff::read(&buf).unwrap() == gff);
}
