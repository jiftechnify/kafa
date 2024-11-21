use crate::{class_file::MethodInfo, vm::frame::Frame};

pub struct Thread {
    frames: Vec<Frame>,
}

impl Thread {
    fn new() -> Thread {
        Thread { frames: Vec::new() }
    }

    fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    fn pop_frame(&mut self) {
        let _ = self.frames.pop().expect("thread frame stack underflow");
    }

    fn current_frame(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }

    fn exec_method(&mut self, method: &MethodInfo) -> Result<(), String> {
        let caller = self.current_frame().expect("caller should be exist");

        // create frame for callee method, and pass arguments from caller's stack
        let mut callee = Frame::new(method);
        Frame::transfer_args(caller, &mut callee, method.num_args());

        // TODO: execute instructions

        Ok(())
    }
}
