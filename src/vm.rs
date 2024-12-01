use std::{fs::File, path::Path, rc::Rc};

use class::{Class, MethodSignature};
use frame::Frame;
use method_area::MethodArea;
use thread::Thread;

use crate::class_file::ClassFile;

mod class;
mod frame;
mod instruction;
mod method_area;
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

        let cls = Rc::new(Class::from_class_file(cls_file));
        let cls_name = cls.name.clone();
        let mut meth_area = MethodArea::with_class(cls);

        // initialize class
        let sig_clinit = MethodSignature::new("<clinit>", "()V");
        let _ = self
            .thread
            .exec_bootstrap_method(&mut meth_area, &cls_name, &sig_clinit);

        let sig = MethodSignature::new(method_name, method_desc);
        self.thread
            .exec_bootstrap_method(&mut meth_area, &cls_name, &sig)?;

        let res = self.thread.current_frame().pop_operand();
        self.thread.pop_frame();
        Ok(res)
    }
}
