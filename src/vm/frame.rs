use std::rc::Rc;

use super::{
    class::{Class, Method, MethodSignature, RunTimeCPInfo},
    value::Value,
};
use crate::support::ByteSeq;

pub struct Frame {
    locals: Vec<Option<Value>>,
    op_stack: Vec<Value>,
    class: Rc<Class>,
    meth_sig: MethodSignature,
    code: ByteSeq,
    pc: u32,
}

impl Frame {
    pub fn new(class: Rc<Class>, method: Rc<Method>) -> Frame {
        let code_reader = ByteSeq::new(method.code.as_slice()).unwrap();

        Frame {
            locals: vec![Option::default(); method.max_locals as usize],
            op_stack: Vec::with_capacity(method.max_stack as usize),
            class,
            meth_sig: method.signature.clone(),
            code: code_reader,
            pc: 0,
        }
    }

    pub fn new_empty() -> Frame {
        Frame {
            locals: Vec::new(),
            op_stack: Vec::new(),
            class: Rc::new(Class::dummy()),
            meth_sig: Default::default(),
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

    pub fn next_param_u32(&mut self) -> u32 {
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

    // 呼び出すメソッドにn個の引数を渡す。staticメソッドの呼び出し時に利用
    //
    // 呼び出し元フレームのスタックトップからn個ぶんの値を、呼び出し先フレームのローカル変数の先頭n個ぶんの値としてセット
    //
    // caller stack:    ..., arg1, arg2, ... , argN (stack top)
    //                         ↓     ↓           ↓
    // callee locals: (head) prm1, prm2, ... , prmN, ...
    pub fn transfer_args(caller: &mut Self, callee: &mut Self, n: usize) {
        callee.set_locals(0, caller.pop_args(n).as_slice());
    }

    // 呼び出すメソッドに、そのメソッドのレシーバ(`this`)とn個の引数を渡す。インスタンスメソッドの呼び出し時に利用
    //
    // レシーバとなる値は、呼び出し元フレームのスタックからすでにpopされている前提
    pub fn transfer_receiver_and_args(caller: &mut Self, callee: &mut Self, this: Value, n: usize) {
        assert!(matches!(this, Value::Reference(_)));
        callee.set_local(0, this);
        callee.set_locals(1, caller.pop_args(n).as_slice());
    }

    fn pop_args(&mut self, n: usize) -> Vec<Option<Value>> {
        let mut args_rev: Vec<Option<Value>> = Vec::new();
        for _ in (0..n).rev() {
            let arg = self.pop_operand();
            match arg {
                // JVMの仕様上、Long/Doubleは連続する2スロットを消費
                // この実装では、1スロット目に実際の値を入れ、2スロット目は空にする
                Value::Long(_) | Value::Double(_) => {
                    args_rev.push(None);
                    args_rev.push(Some(arg));
                }
                _ => args_rev.push(Some(arg)),
            }
        }
        args_rev.into_iter().rev().collect()
    }

    /* Constant Poolの参照 */
    pub fn get_cp_info(&self, idx: u16) -> &RunTimeCPInfo {
        self.class.get_cp_info(idx)
    }
}

impl Frame {
    pub fn executing_method_info(&self) -> String {
        format!("{}.{}", self.class.name, self.meth_sig)
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
