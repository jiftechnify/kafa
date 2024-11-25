use crate::{
    class_file::{ConstantPool, MethodInfo},
    vm::frame::Frame,
};

use super::instruction::exec_instr;

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

    pub fn exec_method(
        &mut self,
        const_pool: ConstantPool,
        method: &MethodInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let caller = self.current_frame();
        let caller_addr = caller as *const Frame as usize;

        // create frame for callee method, and pass arguments from caller's stack
        let mut callee = Frame::new(const_pool, method);
        Frame::transfer_args(caller, &mut callee, method.num_args());

        // switch to the callee frame
        self.push_frame(callee);

        // execute instructions until return
        loop {
            // check if returned from the method called
            let curr_frame_addr = self.current_frame() as *const Frame as usize;
            if caller_addr == curr_frame_addr {
                break;
            }
            exec_instr(self)?;
        }
        Ok(())
    }
}
