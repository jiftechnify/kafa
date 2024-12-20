#![allow(dead_code)]

mod class_file;
mod support;
mod vm;

use std::env;

use vm::{VMResult, Value, VM};

const ENV_KEY_CLASSPATH: &str = "KAFA_CLASSPATH";

fn main() {
    let Ok(cp) = env::var_os(ENV_KEY_CLASSPATH).map_or_else(
        || {
            eprintln!("environment variable {ENV_KEY_CLASSPATH} is not set; fallback to $pwd");
            env::current_dir()
        },
        |cp| Ok(cp.into()),
    ) else {
        eprintln!("failed to determine classpath. abort");
        return;
    };
    println!("classpath: {cp:?}");

    let mut vm = VM::new(&cp);

    print_result(vm.execute("MakeJVM", "start", "()I", &[]));
    print_result(vm.execute("MakeJVM", "start2", "()I", &[]));
    print_result(vm.execute("MakeJVM", "start3", "()Z", &[]));

    print_result(vm.execute("StaticFieldsSample", "start", "()I", &[]));

    print_result(vm.execute("loader/RuntimeClassLoadingSample", "start", "()I", &[]));
}

fn print_result(res: VMResult<Value>) {
    match res {
        Ok(v) => {
            println!("return value: {v:?}");
        }
        Err(e) => {
            println!("failed to execute: {e}");
        }
    }
}
