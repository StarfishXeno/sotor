use std::{fs, io::BufReader, io::prelude::{*}};

mod gff;
mod util;

const PATH: &str = "./save/savenfo.res";
fn main() {
    let gff = gff::read::read(PATH);
    println!("{gff:#?}");
}
