mod class_file;
mod support;
mod vm;

use class_file::ClassFile;
use std::fs::File;

fn main() {
    let mut file = File::open("res/MakeJVM.class").expect("failed to open class file");
    match ClassFile::parse(&mut file) {
        Ok(class_file) => {
            println!("methods:");
            for m in class_file.methods {
                println!("- {}", m);
            }
        }
        Err(e) => println!("failed to parse: {}", e),
    }
}
