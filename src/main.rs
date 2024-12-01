#![allow(dead_code)]

mod class_file;
mod support;
mod vm;

use std::error::Error;

use vm::{Value, VM};

fn main() {
    let mut vm = VM::new();

    print_result(vm.execute("res/MakeJVM.class", "start", "()I", &[]));
    print_result(vm.execute("res/MakeJVM.class", "start2", "()I", &[]));
    print_result(vm.execute("res/MakeJVM.class", "start3", "()Z", &[]));

    print_result(vm.execute("res/StaticFieldsSample.class", "start", "()I", &[]));
}

fn print_result(res: Result<Value, Box<dyn Error>>) {
    match res {
        Ok(v) => {
            println!("return value: {v:?}");
        }
        Err(e) => {
            println!("failed to execute: {e}");
        }
    }
}
