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
        let caller_frame = self.current_frame().expect("caller should be exist");
        let mut callee_frame = Frame::new(method);
        callee_frame.set_locals(&caller_frame.pop_operands(method.num_args()));

        // TODO: execute instructions

        Ok(())
    }
}
