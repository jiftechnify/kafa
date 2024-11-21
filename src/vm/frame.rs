use crate::{class_file::MethodInfo, support::ByteSeq};

pub struct Frame {
    locals: Vec<i32>,
    op_stack: Vec<i32>,
    code: ByteSeq,
    pc: u16,
}

impl Frame {
    pub fn new(method: &MethodInfo) -> Frame {
        let code_attr = method.get_code_attr().expect("method must have code attr");
        let code_reader = ByteSeq::new(code_attr.code.as_slice()).unwrap();

        Frame {
            locals: vec![0i32; code_attr.max_locals as usize],
            op_stack: Vec::new(),
            code: code_reader,
            pc: 0,
        }
    }
}

impl Frame {
    /*  ローカル変数領域操作 */
    fn set_local(&mut self, idx: usize, v: i32) -> &mut Self {
        self.locals[idx] = v;
        self
    }

    fn get_locals(&self) -> &[i32] {
        &self.locals
    }

    /* 命令デコード */
    fn next_instruction(&mut self) -> u8 {
        self.pc = self.code.pos() as u16;
        self.code.read_u8()
    }

    fn next_param_u8(&mut self) -> u8 {
        self.code.read_u8()
    }

    fn next_param_u16(&mut self) -> u16 {
        self.code.read_u16()
    }

    /* プログラムカウンタ操作 */
    fn get_pc(&self) -> u16 {
        self.pc
    }

    fn jump_pc(&mut self, pc: u16) {
        self.pc = pc;
        self.code.seek(pc as usize);
    }

    /* オペランドスタック操作 */
    fn push_operand(&mut self, v: i32) {
        self.op_stack.push(v)
    }

    fn pop_operand(&mut self) -> i32 {
        self.op_stack.pop().expect("stack underflow")
    }

    // 呼び出すメソッドにn個の引数を渡す処理
    // 呼び出し元フレームのスタックトップからn個ぶんの値を、呼び出し先フレームのローカル変数の先頭n個ぶんの値としてセット
    //
    // caller stack:    ..., arg1, arg2, ... , argN (stack top)
    //                         ↓     ↓           ↓
    // callee locals: (head) prm1, prm2, ... , prmN, ...
    pub fn transfer_args(caller: &mut Self, callee: &mut Self, n: usize) {
        for i in (0..n).rev() {
            callee.set_local(i, caller.pop_operand());
        }
    }

    // メソッドの返り値を呼び出し元に返す処理
    // 呼び出されたメソッドのフレームのスタックからpopし、呼び出し元フレームのスタックにpush
    pub fn transfer_ret(caller: &mut Self, callee: &mut Self) {
        caller.push_operand(callee.pop_operand());
    }
}
