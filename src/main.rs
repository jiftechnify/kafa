use std::io::Cursor;

mod class_file;
mod support;

use class_file::ClassFile;

fn main() {
    let mut input = Cursor::new(vec![202u8, 254, 186, 190, 0, 0, 0, 0]);

    match ClassFile::parse(&mut input) {
        Ok(class_file) => println!("parsed class file: {:?}", class_file),
        Err(e) => println!("failed to parse: {}", e),
    }
}
