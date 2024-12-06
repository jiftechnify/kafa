use std::rc::Rc;

use super::{
    class::{Class, MethodSignature},
    error::VMResult,
    frame::Frame,
    heap::Heap,
    instruction::exec_instr,
    method_area::MethodArea,
};

pub struct Thread {
    frames: Vec<Frame>,
}

impl Thread {
    pub fn new() -> Thread {
        Thread { frames: Vec::new() }
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) {
        let _ = self.frames.pop().expect("thread frame stack underflow");
    }

    pub fn current_frame(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("no frame belongs to the thread")
    }

    pub fn exec_bootstrap_method(
        &mut self,
        meth_area: &mut MethodArea,
        heap: &mut Heap,
        class_name: &str,
        sig: &MethodSignature,
    ) -> VMResult<()> {
        let caller = self.current_frame();

        // lookup method to be called
        let cls = meth_area.resolve_class(class_name)?;
        let Some(meth) = cls.lookup_static_method(sig) else {
            return Err(format!("static method {class_name}.{sig} not found"))?;
        };

        // create frame for callee method, and pass arguments from caller's stack
        let num_args = meth.num_args();
        let mut callee = Frame::new(cls, meth);
        Frame::transfer_args(caller, &mut callee, num_args);

        // switch to the callee frame
        self.push_frame(callee);

        // execute instructions until end of program
        loop {
            if self.frames.len() == 1 {
                // returned to bootstrap frame -> program finished!
                break;
            }
            exec_instr(self, meth_area, heap)?;
        }
        Ok(())
    }

    pub(in crate::vm) fn exec_class_initialization(
        &mut self,
        meth_area: &mut MethodArea,
        heap: &mut Heap,
        cls: Rc<Class>,
    ) -> VMResult<()> {
        let sig_clinit = &MethodSignature::new_with_raw_descriptor("<clinit>", "()V");
        let Some(clinit) = cls.lookup_static_method(sig_clinit) else {
            return Ok(());
        };

        let orig_depth = self.frames.len();

        let frame = Frame::new(cls, clinit);
        self.push_frame(frame);

        loop {
            if self.frames.len() == orig_depth {
                break;
            }
            exec_instr(self, meth_area, heap)?;
        }
        Ok(())
    }
}
