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
    0x0A => instr_lconst::<1>,
    0x0B => instr_fconst_0,
    0x0C => instr_fconst_1,
    0x0D => instr_fconst_2,
    0x0E => instr_dconst_0,
    0x0F => instr_dconst_1,
    0x10 => instr_bipush,
    0x11 => instr_sipush,

    0x15 => instr_iload,
    0x16 => instr_lload,
    0x17 => instr_fload,
    0x18 => instr_dload,
    0x19 => instr_aload,
    0x1A => instr_iload_n::<0>,
    0x1B => instr_iload_n::<1>,
    0x1C => instr_iload_n::<2>,
    0x1D => instr_iload_n::<3>,
    0x1E => instr_lload_n::<0>,
    0x1F => instr_lload_n::<1>,
    0x20 => instr_lload_n::<2>,
    0x21 => instr_lload_n::<3>,
    0x22 => instr_fload_n::<0>,
    0x23 => instr_fload_n::<1>,
    0x24 => instr_fload_n::<2>,
    0x25 => instr_fload_n::<3>,
    0x26 => instr_dload_n::<0>,
    0x27 => instr_dload_n::<1>,
    0x28 => instr_dload_n::<2>,
    0x29 => instr_dload_n::<3>,
    0x2A => instr_aload_n::<0>,
    0x2B => instr_aload_n::<1>,
    0x2C => instr_aload_n::<2>,
    0x2D => instr_aload_n::<3>,

    0x36 => instr_istore,
    0x37 => instr_lstore,
    0x38 => instr_fstore,
    0x39 => instr_dstore,
    0x3A => instr_astore,
    0x3B => instr_istore_n::<0>,
    0x3C => instr_istore_n::<1>,
    0x3D => instr_istore_n::<2>,
    0x3E => instr_istore_n::<3>,
    0x3F => instr_lstore_n::<0>,
    0x40 => instr_lstore_n::<1>,
    0x41 => instr_lstore_n::<2>,
    0x42 => instr_lstore_n::<3>,
    0x43 => instr_fstore_n::<0>,
    0x44 => instr_fstore_n::<1>,
    0x45 => instr_fstore_n::<2>,
    0x46 => instr_fstore_n::<3>,
    0x47 => instr_dstore_n::<0>,
    0x48 => instr_dstore_n::<1>,
    0x49 => instr_dstore_n::<2>,
    0x4A => instr_dstore_n::<3>,
    0x4B => instr_astore_n::<0>,
    0x4C => instr_astore_n::<1>,
    0x4D => instr_astore_n::<2>,
    0x4E => instr_astore_n::<3>,

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

// push the specified local (by index) to the operand stack
macro_rules! instr_load {
    ($name:ident, $name_n:ident, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread) -> InstructionResult {
            let frame = t.current_frame();
            let idx = frame.next_param_u8() as usize;
            let v @ $vtype(_) = frame.get_local(idx) else {
                return Err(concat!("target local is not type '", $vtype_name, "'").into());
            };
            frame.push_operand(v);
            Ok(())
        }
        fn $name_n<const N: usize>(t: &mut Thread) -> InstructionResult {
            let frame = t.current_frame();
            let v @ $vtype(_) = frame.get_local(N) else {
                return Err(concat!("target local is not type '", $vtype_name, "'").into());
            };
            frame.push_operand(v);
            Ok(())
        }
    };
}
instr_load!(instr_iload, instr_iload_n, Value::Int, "int");
instr_load!(instr_lload, instr_lload_n, Value::Long, "long");
instr_load!(instr_fload, instr_fload_n, Value::Float, "float");
instr_load!(instr_dload, instr_dload_n, Value::Double, "double");
instr_load!(instr_aload, instr_aload_n, Value::Reference, "reference");

// pop from the operand stack and store it to the specified local (by index)
macro_rules! instr_store {
    ($name:ident, $name_n:ident, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread) -> InstructionResult {
            let frame = t.current_frame();
            let idx = frame.next_param_u8() as usize;
            let v @ $vtype(_) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            frame.set_local(idx, v);
            Ok(())
        }
        fn $name_n<const N: usize>(t: &mut Thread) -> InstructionResult {
            let frame = t.current_frame();
            let v @ $vtype(_) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            frame.set_local(N, v);
            Ok(())
        }
    };
}
instr_store!(instr_istore, instr_istore_n, Value::Int, "int");
instr_store!(instr_lstore, instr_lstore_n, Value::Long, "long");
instr_store!(instr_fstore, instr_fstore_n, Value::Float, "float");
instr_store!(instr_dstore, instr_dstore_n, Value::Double, "double");
instr_store!(instr_astore, instr_astore_n, Value::Reference, "reference");

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
