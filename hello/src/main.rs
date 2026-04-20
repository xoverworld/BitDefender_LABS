use std::env;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

fn main(){
    let args: Vec<String> = env::args().collect();

    let aux = &args[1];
    let path = Path::new(aux);
    println!("path: {}",path.display());
    let zip_file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    list_zip_contents(zip_file);
}

fn list_zip_contents(reader: impl Read + Seek) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipArchive::new(reader)?;
    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        if file.is_file() {
            println!("{}", file.name());
        }
    }
    Ok(())
}