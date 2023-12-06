use std::{fs, io::BufReader, io::prelude::{*}};

mod gff;
mod util;

const PATH: &str = "./save/globalvars.res";
fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let gff = gff::read::read(PATH);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    //println!("{gff:#?}");
}
