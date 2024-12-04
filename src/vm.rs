mod class;
mod class_loader;
mod frame;
mod heap;
mod instruction;
mod method_area;
mod thread;
mod value;

use std::ffi::{OsStr, OsString};

use class::MethodSignature;
pub use class_loader::ClassLoader;
use frame::Frame;
use method_area::MethodArea;
use thread::Thread;
pub use value::Value;

pub struct VM {
    thread: Thread,
    classpath: OsString,
}

impl VM {
    pub fn new<P>(classpath: &P) -> VM
    where
        P: AsRef<OsStr>,
    {
        VM {
            thread: Thread::new(),
            classpath: OsString::from(classpath),
        }
    }

    pub fn execute(
        &mut self,
        class_name: &str,
        method_name: &str,
        method_desc: &str,
        args: &[Value],
    ) -> Result<Value, Box<dyn std::error::Error>> {
        println!("executing {class_name}.{method_name}:{method_desc} with args: {args:?}");

        let cls_loader = ClassLoader::new(&self.classpath);
        let mut meth_area = MethodArea::new(cls_loader);

        // initialize class
        let init_cls = meth_area.lookup_class(class_name)?;
        init_cls.initialize(&mut self.thread, &mut meth_area)?;

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
