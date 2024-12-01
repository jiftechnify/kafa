use super::{
    class::MethodSignature, frame::Frame, instruction::exec_instr, method_area::MethodArea,
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

    pub fn invoke_static_method(
        &mut self,
        meth_area: &mut MethodArea,
        class_name: &str,
        sig: &MethodSignature,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let caller_depth = self.frames.len();

        let caller = self.current_frame();

        // lookup method to be called
        let Some(cls) = meth_area.lookup_class(class_name) else {
            return Err(format!("class '{class_name}' not found"))?;
        };
        let Some(meth) = cls.lookup_static_method(sig) else {
            return Err(format!("static method {class_name}.{sig} not found"))?;
        };

        // create frame for callee method, and pass arguments from caller's stack
        let num_args = meth.num_args();
        let mut callee = Frame::new(cls, meth);
        Frame::transfer_args(caller, &mut callee, num_args);

        // switch to the callee frame
        self.push_frame(callee);

        // execute instructions until return
        loop {
            // check if returned from the method called
            let curr_frame_depth = self.frames.len();
            if caller_depth == curr_frame_depth {
                break;
            }
            exec_instr(self, meth_area)?;
        }
        Ok(())
    }
}
