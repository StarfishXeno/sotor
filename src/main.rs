use crate::{gff::FieldValue, util::read_file};

mod gff;
mod util;

const GLOBALS_PATH: &str = "./save/globalvars.res";
const NFO_PATH: &str = "./save/savenfo.res";
fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let globals_bytes = read_file(GLOBALS_PATH).unwrap();
    let globals = gff::read(&globals_bytes);
    if let Ok(gff) = globals {
        let keys = gff.content.fields.keys();
        println!("Keys: {:?}", keys);
        for key in keys {
            let val = gff.content.fields.get(key).unwrap();
            if let FieldValue::List(val) = val {
                println!("{key}: {} items", val.len());
            } else if let FieldValue::Void(bytes) = val {
                println!("{key}: {} items", bytes.len());
            }
        }
    } else {
        println!("{globals:?}")
    }
    println!("//////////////////");
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
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
