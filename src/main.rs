#![allow(dead_code)]

mod class_file;
mod support;
mod vm;

use vm::VM;

fn main() {
    let mut vm = VM::new();
    let res = vm.execute("res/MakeJVM.class", "compute", "(I)I", &[10]);
    match res {
        Ok(v) => {
            println!("return value: {v}");
        }
        Err(e) => {
            println!("failed to execute: {e}");
        }
    }
}
