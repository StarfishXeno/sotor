use crate::{
    formats::{erf, gff},
    util::{read_file, get_erf_date},
};

mod formats;
mod util;

const ERF_PATH: &str = "./save/savegame.sav";
fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let (build_year, build_day) = get_erf_date();
    let erf_bytes = read_file(ERF_PATH).unwrap();
    let mut erf = erf::read(&erf_bytes).unwrap();
    println!("{erf:#?}");
    erf.build_year = build_year;
    erf.build_day = build_day;
    let erf_buf = erf::write(erf.clone());
    
    let res = erf.resources.get("pc").unwrap();
    let gff = gff::read(&res.content).unwrap();
    println!("{:#?}", gff.content.fields.keys());
    let gff_buf = gff::write(gff.clone());
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    
    assert!(gff::read(&gff_buf).unwrap() == gff);
    assert!(erf::read(&erf_buf).unwrap() == erf);
}
