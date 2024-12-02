#![allow(dead_code)]

mod class_file;
mod support;
mod vm;

use std::error::Error;

use vm::{Value, VM};

fn main() {
    let Some(cp) = std::env::var_os("KAFA_CLASS_PATH") else {
        eprintln!("environment variable KAFA_CLASS_PATH is not defined");
        return;
    };
    println!("class path: {cp:?}");

    let mut vm = VM::new(&cp);

    print_result(vm.execute("MakeJVM", "start", "()I", &[]));
    print_result(vm.execute("MakeJVM", "start2", "()I", &[]));
    print_result(vm.execute("MakeJVM", "start3", "()Z", &[]));

    print_result(vm.execute("StaticFieldsSample", "start", "()I", &[]));

    print_result(vm.execute("loader/RuntimeClassLoadingSample", "start", "()I", &[]));
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
