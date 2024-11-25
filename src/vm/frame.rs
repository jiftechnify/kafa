use super::value::Value;
use crate::{
    class_file::{CPInfo, ConstantPool, MethodInfo},
    support::ByteSeq,
};

pub struct Frame {
    locals: Vec<Option<Value>>,
    op_stack: Vec<Value>,
    const_pool: ConstantPool,
    code: ByteSeq,
    pc: u32,
}

impl Frame {
    pub fn new(const_pool: ConstantPool, method: &MethodInfo) -> Frame {
        let code_attr = method.get_code_attr().expect("method must have code attr");
        let code_reader = ByteSeq::new(code_attr.code.as_slice()).unwrap();

        Frame {
            locals: vec![Option::default(); code_attr.max_locals as usize],
            op_stack: Vec::with_capacity(code_attr.max_stack as usize),
            const_pool,
            code: code_reader,
            pc: 0,
        }
    }

    pub fn new_empty() -> Frame {
        Frame {
            locals: Vec::new(),
            op_stack: Vec::new(),
            const_pool: ConstantPool::empty(),
            code: ByteSeq::new(vec![].as_slice()).unwrap(),
            pc: 0,
        }
    }
}

impl Frame {
    /*  ローカル変数領域操作 */
    pub fn set_local(&mut self, idx: usize, v: Value) -> &mut Self {
        self.locals[idx] = Some(v);
        self
    }

    fn set_locals(&mut self, idx: usize, vs: &[Option<Value>]) {
        self.locals[idx..(vs.len() + idx)].copy_from_slice(vs);
    }

    pub fn get_local(&self, idx: usize) -> Value {
        self.locals[idx].expect("local not exist")
    }

    fn get_locals(&self) -> &[Option<Value>] {
        &self.locals
    }

    /* 命令デコード */
    pub fn next_instruction(&mut self) -> u8 {
        self.pc = self.code.pos() as u32;
        self.code.read_u8()
    }

    pub fn next_param_u8(&mut self) -> u8 {
        self.code.read_u8()
    }

    pub fn next_param_u16(&mut self) -> u16 {
        self.code.read_u16()
    }

    pub fn next_params_u32(&mut self) -> u32 {
        self.code.read_u32()
    }

    // n-byteアラインメントのためのパディングを読み飛ばす
    pub fn skip_code_padding(&mut self, align: usize) {
        let pad_size = align - self.code.pos() % align;
        if pad_size == align {
            // already aligned
            return;
        }
        self.code.skip(pad_size);
    }

    /* プログラムカウンタ操作 */
    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    pub fn jump_pc(&mut self, pc: u32) {
        self.pc = pc;
        self.code.seek(pc as usize);
    }

    /* オペランドスタック操作 */
    pub fn push_operand(&mut self, v: Value) {
        self.op_stack.push(v)
    }

    pub fn pop_operand(&mut self) -> Value {
        self.op_stack.pop().expect("stack underflow")
    }

    pub fn peek_operand(&self) -> &Value {
        self.op_stack.last().expect("stack underflow")
    }

    pub fn dup_operand(&mut self) {
        let v = self.peek_operand();
        self.push_operand(*v);
    }

    // 呼び出すメソッドにn個の引数を渡す処理
    // 呼び出し元フレームのスタックトップからn個ぶんの値を、呼び出し先フレームのローカル変数の先頭n個ぶんの値としてセット
    //
    // caller stack:    ..., arg1, arg2, ... , argN (stack top)
    //                         ↓     ↓           ↓
    // callee locals: (head) prm1, prm2, ... , prmN, ...
    pub fn transfer_args(caller: &mut Self, callee: &mut Self, n: usize) {
        let mut locals_rev: Vec<Option<Value>> = Vec::new();
        for _ in (0..n).rev() {
            let arg = caller.pop_operand();
            match arg {
                // JVMの仕様上、Long/Doubleは連続する2スロットを消費
                // この実装では、1スロット目に実際の値を入れ、2スロット目は空にする
                Value::Long(_) | Value::Double(_) => {
                    locals_rev.push(None);
                    locals_rev.push(Some(arg));
                }
                _ => locals_rev.push(Some(arg)),
            }
        }
        callee.set_locals(
            0,
            locals_rev.into_iter().rev().collect::<Vec<_>>().as_slice(),
        );
    }

    pub fn get_cp_info(&self, idx: u16) -> &CPInfo {
        self.const_pool.get_info(idx)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transfer_args() {
        use Value::*;

        let mut caller = Frame::new_empty();
        let mut callee = Frame::new_empty();

        caller.push_operand(Int(1));
        caller.push_operand(Long(1));
        caller.push_operand(Float(1.0));
        caller.push_operand(Double(1.0));

        Frame::transfer_args(&mut caller, &mut callee, 4);

        assert_eq!(
            callee.get_locals(),
            &vec![
                Some(Int(1)),
                Some(Long(1)),
                None,
                Some(Float(1.0)),
                Some(Double(1.0)),
                None,
            ]
        )
    }
}
