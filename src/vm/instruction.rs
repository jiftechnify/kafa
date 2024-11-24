use super::thread::Thread;
use super::value::Value;

type InstructionResult = Result<(), Box<dyn std::error::Error>>;
type Instruction = fn(&mut Thread) -> InstructionResult;

pub fn exec_instr(thread: &mut Thread) -> InstructionResult {
    let op_code = thread.current_frame().next_instruction();
    let instr = INSTRUCTION_TABLE[op_code as usize]
        .ok_or_else(|| format!("op(code = {:#x}) has been not implemented", op_code))?;

    instr(thread)
}

macro_rules! instruction_table {
    ($($op_code:expr => $instr_impl:expr$(,)?),*) => {
        {
            let mut table: [Option<Instruction>; 256] = [None; 256];
            $(table[$op_code] = Some($instr_impl);)*
            table
        }
    };
}

const INSTRUCTION_TABLE: [Option<Instruction>; 256] = instruction_table! {
    0x00 => instr_nop,

    0x02 => instr_iconst::<-1>,
    0x03 => instr_iconst::<0>,
    0x04 => instr_iconst::<1>,
    0x05 => instr_iconst::<2>,
    0x06 => instr_iconst::<3>,
    0x07 => instr_iconst::<4>,
    0x08 => instr_iconst::<5>,
    0x09 => instr_lconst::<0>,
    0x10 => instr_lconst::<1>,
    0x11 => instr_fconst_0,
    0x12 => instr_fconst_1,
    0x13 => instr_fconst_2,
    0x14 => instr_dconst_0,
    0x15 => instr_dconst_1,
    0x16 => instr_bipush,
    0x17 => instr_sipush,

    0x1A => instr_iload::<0>,
    0x1B => instr_iload::<1>,
    0x1C => instr_iload::<2>,
    0x3C => instr_istore::<1>,
    0x3D => instr_istore::<2>,
    0x60 => instr_iadd,
    0x84 => instr_iinc,
    0xA3 => instr_if_icmpgt,
    0xA7 => instr_goto,
    0xAC => instr_ireturn,
};

// no-op
fn instr_nop(_: &mut Thread) -> InstructionResult {
    Ok(())
}

// push the constant N to the operand stack (int)
fn instr_iconst<const N: i32>(t: &mut Thread) -> InstructionResult {
    t.current_frame().push_operand(Value::Int(N));
    Ok(())
}

// push the constant N to the operand stack (long)
fn instr_lconst<const N: i64>(t: &mut Thread) -> InstructionResult {
    t.current_frame().push_operand(Value::Long(N));
    Ok(())
}

// push the constant N to the operand stack (float)
macro_rules! instr_fconst {
    ($name:ident, $value:expr) => {
        fn $name(t: &mut Thread) -> InstructionResult {
            t.current_frame().push_operand(Value::Float($value));
            Ok(())
        }
    };
}
instr_fconst!(instr_fconst_0, 0.0);
instr_fconst!(instr_fconst_1, 1.0);
instr_fconst!(instr_fconst_2, 2.0);

// push the constant N to the operand stack (double)
macro_rules! instr_dconst {
    ($name:ident, $value:expr) => {
        fn $name(t: &mut Thread) -> InstructionResult {
            t.current_frame().push_operand(Value::Double($value));
            Ok(())
        }
    };
}
instr_dconst!(instr_dconst_0, 0.0);
instr_dconst!(instr_dconst_1, 1.0);

// push immediate byte to the operand stack (byte is sign-extended to an int value)
fn instr_bipush(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();
    let v = frame.next_param_u8() as i8 as i32;
    frame.push_operand(Value::Int(v));
    Ok(())
}

// push immediate short to the operand stack (short is sign-extended to an int value)
fn instr_sipush(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();
    let v = frame.next_param_u16() as i16 as i32;
    frame.push_operand(Value::Int(v));
    Ok(())
}

// push the Nth local to the operand stack
fn instr_iload<const N: usize>(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();
    frame.push_operand(frame.get_local(N));
    Ok(())
}

// pop from the operand stack and store it to the Nth local
fn instr_istore<const N: usize>(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();
    let v = frame.pop_operand();
    frame.set_local(N, v);
    Ok(())
}

// pop 2 values, add them and push the result
fn instr_iadd(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();
    let Value::Int(rhs) = frame.pop_operand() else {
        return Err("target operand is not type 'int'".into());
    };
    let Value::Int(lhs) = frame.pop_operand() else {
        return Err("target operand is not type 'int'".into());
    };
    frame.push_operand(Value::Int(lhs + rhs));
    Ok(())
}

// increment the value of the local (specified by index) by delta
// operands: target local index, delta(signed int)
#[allow(overflowing_literals)]
fn instr_iinc(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();

    let idx = frame.next_param_u8() as usize;
    let delta = frame.next_param_u8() as i8 as i32;
    let Value::Int(v) = frame.get_local(idx) else {
        return Err("target local is not type 'int'".into());
    };
    frame.set_local(idx, Value::Int(v + delta));
    Ok(())
}

// compare the top (rhs) and the 2nd-top (lhs) values of the operand stack.
// if lhs > rhs, move PC to: {current PC} + {delta}
// operands: delta of PC(signed int)
#[allow(overflowing_literals)]
fn instr_if_icmpgt(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();

    let pc_delta = frame.next_param_u16() as i16;
    let Value::Int(rhs) = frame.pop_operand() else {
        return Err("target operand is not type 'int'".into());
    };
    let Value::Int(lhs) = frame.pop_operand() else {
        return Err("target operand is not type 'int'".into());
    };
    if lhs > rhs {
        let jmp_dest = (frame.get_pc() as i16 + pc_delta) as u16;
        frame.jump_pc(jmp_dest);
    }
    Ok(())
}

// move PC to: {current PC} + {delta}
// operands: delta of PC(signed int)
fn instr_goto(t: &mut Thread) -> InstructionResult {
    let frame = t.current_frame();

    let pc_delta = frame.next_param_u16() as i16;
    let jmp_dest = (frame.get_pc() as i16 + pc_delta) as u16;
    frame.jump_pc(jmp_dest);
    Ok(())
}

// return from the method
fn instr_ireturn(t: &mut Thread) -> InstructionResult {
    // pop from the operand stack; it's a return value of the method
    let ret = t.current_frame().pop_operand();
    // discard the frame for the method
    t.pop_frame();

    // now, current_frame is the frame for the callee method
    // push the return value to the operand stack of the callee frame
    t.current_frame().push_operand(ret);
    Ok(())
}
