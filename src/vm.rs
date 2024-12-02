use std::ffi::{OsStr, OsString};

use class::MethodSignature;
use class_loader::ClassLoader;
use frame::Frame;
use method_area::MethodArea;
use thread::Thread;

mod class;
mod class_loader;
mod frame;
mod instruction;
mod method_area;
mod thread;
mod value;

pub use value::Value;

pub struct VM {
    thread: Thread,
    class_path: OsString,
}

impl VM {
    pub fn new<P>(class_path: &P) -> VM
    where
        P: AsRef<OsStr>,
    {
        VM {
            thread: Thread::new(),
            class_path: OsString::from(class_path),
        }
    }

    pub fn execute(
        &mut self,
        class_name: &str,
        method_name: &str,
        method_desc: &str,
        args: &[Value],
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let cls_loader = ClassLoader::new(&self.class_path);
        let mut meth_area = MethodArea::new(cls_loader);

        // initialize class
        let clinit_frame = Frame::new_empty();
        self.thread.push_frame(clinit_frame);

        let sig_clinit = MethodSignature::new("<clinit>", "()V");
        let _ = self
            .thread
            .exec_bootstrap_method(&mut meth_area, class_name, &sig_clinit);
        self.thread.pop_frame();

        // execute bootstrap method
        let mut bs_frame = Frame::new_empty();
        for arg in args {
            bs_frame.push_operand(*arg);
        }
        self.thread.push_frame(bs_frame);

        let sig = MethodSignature::new(method_name, method_desc);
        self.thread
            .exec_bootstrap_method(&mut meth_area, class_name, &sig)?;

        let res = self.thread.current_frame().pop_operand();
        self.thread.pop_frame();
        Ok(res)
    }
}
