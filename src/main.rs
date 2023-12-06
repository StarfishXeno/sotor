use crate::gff::FieldValue;

mod gff;
mod util;

const GLOBALS_PATH: &str = "./save/globalvars.res";
const NFO_PATH: &str = "./save/savenfo.res";
fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let globals = gff::read::read(GLOBALS_PATH);
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
    let nfo = gff::read::read(NFO_PATH);
    if let Ok(gff) = nfo {
        println!("{:#?}", gff.content.fields);
    } else {
        println!("{nfo:?}")
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}
