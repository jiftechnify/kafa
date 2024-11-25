use std::{fs::File, path::Path};

use frame::Frame;
use thread::Thread;

use crate::class_file::ClassFile;

mod frame;
mod instruction;
mod thread;
mod value;

pub use value::Value;

pub struct VM {
    thread: Thread,
}

impl VM {
    pub fn new() -> VM {
        VM {
            thread: Thread::new(),
        }
    }

    pub fn execute<P>(
        &mut self,
        class_file_path: P,
        method_name: &str,
        method_desc: &str,
        args: &[Value],
    ) -> Result<Value, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let f = File::open(class_file_path)?;
        let cls_file = ClassFile::parse(f)?;

        let mut init_frame = Frame::new_empty();
        for arg in args {
            init_frame.push_operand(*arg);
        }
        self.thread.push_frame(init_frame);

        let method = cls_file
            .find_method(method_name, method_desc)
            .ok_or_else(|| format!("method not found (name={method_name}, desc={method_desc})"))?;

        self.thread
            .exec_method(cls_file.constant_pool.clone(), method)?;

        let res = self.thread.current_frame().pop_operand();
        Ok(res)
    }
}
