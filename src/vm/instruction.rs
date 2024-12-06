use crate::vm::heap::RefValue;

use super::class::{MethodSignature, RunTimeCPInfo as CPInfo};
use super::frame::Frame;
use super::heap::Heap;
use super::method_area::MethodArea;
use super::thread::Thread;
use super::value::{Value, ValueCategory};

type InstructionResult = Result<(), Box<dyn std::error::Error>>;
type Instruction = fn(&mut Thread, &mut MethodArea, &mut Heap) -> InstructionResult;

pub fn exec_instr(
    thread: &mut Thread,
    meth_area: &mut MethodArea,
    heap: &mut Heap,
) -> InstructionResult {
    let op_code = thread.current_frame().next_instruction();
    let instr = INSTRUCTION_TABLE[op_code as usize]
        .ok_or_else(|| format!("op(code = {:#x}) has been not implemented", op_code))?;

    instr(thread, meth_area, heap)
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

    0x01 => instr_aconst_null,
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
    0x12 => instr_ldc,
    0x13 => instr_ldc_w,
    0x14 => instr_ldc2_w,

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
    0x2E => instr_iaload,
    0x2F => instr_laload,
    0x30 => instr_faload,
    0x31 => instr_daload,
    0x32 => instr_aaload,
    0x33 => instr_baload,
    0x34 => instr_caload,
    0x35 => instr_saload,

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
    0x4F => instr_iastore,
    0x50 => instr_lastore,
    0x51 => instr_fastore,
    0x52 => instr_dastore,
    0x53 => instr_aastore,
    0x54 => instr_bastore,
    0x55 => instr_castore,
    0x56 => instr_sastore,

    0x57 => instr_pop,
    0x58 => instr_pop2,
    0x59 => instr_dup,
    0x5A => instr_dup_x1,
    0x5B => instr_dup_x2,
    0x5C => instr_dup2,
    0x5D => instr_dup2_x1,
    0x5E => instr_dup2_x2,
    0x5F => instr_swap,

    0x60 => instr_iadd,
    0x61 => instr_ladd,
    0x62 => instr_fadd,
    0x63 => instr_dadd,
    0x64 => instr_isub,
    0x65 => instr_lsub,
    0x66 => instr_fsub,
    0x67 => instr_dsub,
    0x68 => instr_imul,
    0x69 => instr_lmul,
    0x6A => instr_fmul,
    0x6B => instr_dmul,
    0x6C => instr_idiv,
    0x6D => instr_ldiv,
    0x6E => instr_fdiv,
    0x6F => instr_ddiv,
    0x70 => instr_irem,
    0x71 => instr_lrem,
    0x72 => instr_frem,
    0x73 => instr_drem,
    0x74 => instr_ineg,
    0x75 => instr_lneg,
    0x76 => instr_fneg,
    0x77 => instr_dneg,
    0x78 => instr_ishl,
    0x79 => instr_lshl,
    0x7A => instr_ishr,
    0x7B => instr_lshr,
    0x7C => instr_iushr,
    0x7D => instr_lushr,
    0x7E => instr_iand,
    0x7F => instr_land,
    0x80 => instr_ior,
    0x81 => instr_lor,
    0x82 => instr_ixor,
    0x83 => instr_lxor,
    0x84 => instr_iinc,

    0x85 => instr_i2l,
    0x86 => instr_i2f,
    0x87 => instr_i2d,
    0x88 => instr_l2i,
    0x89 => instr_l2f,
    0x8A => instr_l2d,
    0x8B => instr_f2i,
    0x8C => instr_f2l,
    0x8D => instr_f2d,
    0x8E => instr_d2i,
    0x8F => instr_d2l,
    0x90 => instr_d2f,
    0x91 => instr_i2b,
    0x92 => instr_i2c,
    0x93 => instr_i2s,

    0x94 => instr_lcmp,
    0x95 => instr_fcmpl,
    0x96 => instr_fcmpg,
    0x97 => instr_dcmpl,
    0x98 => instr_dcmpg,
    0x99 => instr_ifeq,
    0x9A => instr_ifne,
    0x9B => instr_iflt,
    0x9C => instr_ifge,
    0x9D => instr_ifgt,
    0x9E => instr_ifle,
    0x9F => instr_if_icmpeq,
    0xA0 => instr_if_icmpne,
    0xA1 => instr_if_icmplt,
    0xA2 => instr_if_icmpge,
    0xA3 => instr_if_icmpgt,
    0xA4 => instr_if_icmple,
    0xA5 => instr_if_acmpeq,
    0xA6 => instr_if_acmpne,

    0xA7 => instr_goto,
    0xA8 => instr_jsr,
    0xA9 => instr_ret,
    0xAA => instr_tableswitch,
    0xAB => instr_lookupswitch,
    0xAC => instr_ireturn,
    0xAD => instr_lreturn,
    0xAE => instr_freturn,
    0xAF => instr_dreturn,
    0xB0 => instr_areturn,
    0xB1 => instr_return,

    // TODO: implement instructions for reference type values
    0xB2 => instr_getstatic,
    0xB3 => instr_putstatic,
    0xB4 => instr_getfield,
    0xB5 => instr_putfield,
    0xB6 => instr_invokevirtual,
    0xB7 => instr_invokespecial,
    0xB8 => instr_invokestatic,
    // 0xB9 => instr_invokeinterface,
    // 0xBA => instr_invokedynamic,
    0xBB => instr_new,
    0xBC => instr_newarray,
    0xBD => instr_anewarray,
    0xBE => instr_arraylength,
    // 0xBF => instr_athrow,
    // 0xC0 => instr_checkcast,
    // 0xC1 => instr_instanceof,
};

// no-op
fn instr_nop(_: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    Ok(())
}

// push null reference to the operand stack
// TODO: representation of null reference is TENTATIVE
fn instr_aconst_null(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    t.current_frame().push_operand(Value::Reference(0));
    Ok(())
}

// push the constant N to the operand stack (int)
fn instr_iconst<const N: i32>(
    t: &mut Thread,
    _: &mut MethodArea,
    _: &mut Heap,
) -> InstructionResult {
    t.current_frame().push_operand(Value::Int(N));
    Ok(())
}

// push the constant N to the operand stack (long)
fn instr_lconst<const N: i64>(
    t: &mut Thread,
    _: &mut MethodArea,
    _: &mut Heap,
) -> InstructionResult {
    t.current_frame().push_operand(Value::Long(N));
    Ok(())
}

// push the constant N to the operand stack (float)
macro_rules! instr_fconst {
    ($name:ident, $value:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
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
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            t.current_frame().push_operand(Value::Double($value));
            Ok(())
        }
    };
}
instr_dconst!(instr_dconst_0, 0.0);
instr_dconst!(instr_dconst_1, 1.0);

// push immediate byte to the operand stack (byte is sign-extended to an int value)
fn instr_bipush(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let v = frame.next_param_u8() as i8 as i32;
    frame.push_operand(Value::Int(v));
    Ok(())
}

// push immediate short to the operand stack (short is sign-extended to an int value)
fn instr_sipush(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let v = frame.next_param_u16() as i16 as i32;
    frame.push_operand(Value::Int(v));
    Ok(())
}

// push a constant from constant pool to the operand stack
fn instr_ldc(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let idx = frame.next_param_u8();
    let v = match frame.get_cp_info(idx as u16) {
        CPInfo::Integer(v) => Value::Int(*v),
        CPInfo::Float(v) => Value::Float(*v),
        CPInfo::Double(_) | CPInfo::Long(_) => Err("can't load double/long with ldc")?,
        // TODO: support symbolic references, string consts, etc.
        _ => Err("unsupported constant pool entry")?,
    };
    frame.push_operand(v);
    Ok(())
}

// push a constant from constant pool to the operand stack (wide index)
fn instr_ldc_w(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let idx = frame.next_param_u16();
    let v = match frame.get_cp_info(idx) {
        CPInfo::Integer(v) => Value::Int(*v),
        CPInfo::Float(v) => Value::Float(*v),
        CPInfo::Double(_) | CPInfo::Long(_) => Err("can't load double/long consts with ldc_w")?,
        // TODO: support symbolic references, string consts, etc.
        _ => Err("unsupported constant pool entry")?,
    };
    frame.push_operand(v);
    Ok(())
}

// push a long/double constant from constant pool to the operand stack (wide index)
fn instr_ldc2_w(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let idx = frame.next_param_u16();
    let v = match frame.get_cp_info(idx) {
        CPInfo::Long(v) => Value::Long(*v),
        CPInfo::Double(v) => Value::Double(*v),
        CPInfo::Integer(_) | CPInfo::Float(_) => Err("can't load int/float consts with ldc2_w")?,
        // TODO: support symbolic references
        _ => Err("unsupported constant pool entry")?,
    };
    frame.push_operand(v);
    Ok(())
}

// push the specified local (by index) to the operand stack
macro_rules! instr_load {
    ($name:ident, $name_n:ident, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let idx = frame.next_param_u8() as usize;
            let v @ $vtype(_) = frame.get_local(idx) else {
                return Err(concat!("target local is not type '", $vtype_name, "'").into());
            };
            frame.push_operand(v);
            Ok(())
        }
        fn $name_n<const N: usize>(
            t: &mut Thread,
            _: &mut MethodArea,
            _: &mut Heap,
        ) -> InstructionResult {
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

// push the an element of an array to the operand stack
macro_rules! instr_aload {
    ($name:ident, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, heap: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();

            let Value::Int(idx) = frame.pop_operand() else {
                return Err("index is not an int")?;
            };

            let Value::Reference(r) = frame.pop_operand() else {
                return Err("operand is not a reference value")?;
            };
            let Some(rv) = heap.get(r) else {
                return Err("referent not found on heap")?;
            };
            let RefValue::Array(arr) = rv else {
                return Err("referent is not an array")?;
            };

            let Some(v) = arr.get(idx as usize) else {
                return Err("array index out of bound")?;
            };
            let v @ $vtype(_) = v else {
                return Err(concat!("got value doesn't have type '", $vtype_name, "'"))?;
            };
            frame.push_operand(v);

            Ok(())
        }
    };
}

instr_aload!(instr_iaload, Value::Int, "int");
instr_aload!(instr_laload, Value::Long, "long");
instr_aload!(instr_faload, Value::Float, "float");
instr_aload!(instr_daload, Value::Double, "double");
instr_aload!(instr_aaload, Value::Reference, "reference");
instr_aload!(instr_baload, Value::Byte, "byte");
instr_aload!(instr_caload, Value::Char, "char");
instr_aload!(instr_saload, Value::Short, "short");

// pop from the operand stack and store it to the specified local (by index)
macro_rules! instr_store {
    ($name:ident, $name_n:ident, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let idx = frame.next_param_u8() as usize;
            let v @ $vtype(_) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            frame.set_local(idx, v);
            Ok(())
        }
        fn $name_n<const N: usize>(
            t: &mut Thread,
            _: &mut MethodArea,
            _: &mut Heap,
        ) -> InstructionResult {
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

macro_rules! instr_astore {
    ($name:ident, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, heap: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();

            let v @ $vtype(_) = frame.pop_operand() else {
                return Err(concat!(
                    "the value to store doesn't have type '",
                    $vtype_name,
                    "'"
                ))?;
            };

            let Value::Int(idx) = frame.pop_operand() else {
                return Err("index is not an int")?;
            };

            let Value::Reference(r) = frame.pop_operand() else {
                return Err("operand is not a reference value")?;
            };
            let Some(rv) = heap.get(r) else {
                return Err("referent not found on heap")?;
            };
            let RefValue::Array(arr) = rv else {
                return Err("referent is not an array")?;
            };

            arr.put(idx as usize, v);

            Ok(())
        }
    };
}

instr_astore!(instr_iastore, Value::Int, "int");
instr_astore!(instr_lastore, Value::Long, "long");
instr_astore!(instr_fastore, Value::Float, "float");
instr_astore!(instr_dastore, Value::Double, "double");
instr_astore!(instr_aastore, Value::Reference, "reference");
instr_astore!(instr_bastore, Value::Byte, "byte");
instr_astore!(instr_castore, Value::Char, "char");
instr_astore!(instr_sastore, Value::Short, "short");

macro_rules! pop_operand_if_category_matches {
    ($frame:expr, $category:pat) => {{
        let $category = $frame.peek_operand().category() else {
            return Err("can't execute instruction to current operand stack".into());
        };
        $frame.pop_operand()
    }};
}

fn instr_pop(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    pop_operand_if_category_matches!(frame, ValueCategory::One);
    Ok(())
}

fn instr_pop2(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    match frame.peek_operand().category() {
        ValueCategory::Two => {
            frame.pop_operand();
            Ok(())
        }
        ValueCategory::One => {
            frame.pop_operand();
            pop_operand_if_category_matches!(frame, ValueCategory::One);
            Ok(())
        }
    }
}

fn instr_dup(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let top = frame.peek_operand();
    let ValueCategory::One = top.category() else {
        return Err("top of operand stack is not category 1 value".into());
    };
    frame.dup_operand();
    Ok(())
}

fn instr_dup_x1(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    let v1 = pop_operand_if_category_matches!(frame, ValueCategory::One);
    let v2 = pop_operand_if_category_matches!(frame, ValueCategory::One);

    frame.push_operand(v1);
    frame.push_operand(v2);
    frame.push_operand(v1);
    Ok(())
}

fn instr_dup_x2(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let v1 = pop_operand_if_category_matches!(frame, ValueCategory::One);

    match frame.peek_operand().category() {
        ValueCategory::One => {
            // Form 1 (..., v3, v2, v1 -> ..., v1, v3, v2, v1)
            let v2 = frame.pop_operand();
            let v3 = pop_operand_if_category_matches!(frame, ValueCategory::One);

            frame.push_operand(v1);
            frame.push_operand(v3);
            frame.push_operand(v2);
            frame.push_operand(v1);
            Ok(())
        }
        ValueCategory::Two => {
            // Form 2 (..., v2, v1 -> ..., v1, v2, v1 )
            let v2 = frame.pop_operand();
            frame.push_operand(v1);
            frame.push_operand(v2);
            frame.push_operand(v1);
            Ok(())
        }
    }
}

fn instr_dup2(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    match frame.peek_operand().category() {
        ValueCategory::One => {
            // Form 1 (..., v2, v1 -> ..., v2, v1, v2, v1)
            let v1 = frame.pop_operand();
            let v2 = pop_operand_if_category_matches!(frame, ValueCategory::One);

            frame.push_operand(v2);
            frame.push_operand(v1);
            frame.push_operand(v2);
            frame.push_operand(v1);
            Ok(())
        }
        ValueCategory::Two => {
            // Form 2 (..., v1 -> ..., v1, v1)
            frame.dup_operand();
            Ok(())
        }
    }
}

fn instr_dup2_x1(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    match frame.peek_operand().category() {
        ValueCategory::One => {
            // Form 1
            let v1 = frame.pop_operand();
            let v2 = pop_operand_if_category_matches!(frame, ValueCategory::One);
            let v3 = pop_operand_if_category_matches!(frame, ValueCategory::One);

            frame.push_operand(v2);
            frame.push_operand(v1);
            frame.push_operand(v3);
            frame.push_operand(v2);
            frame.push_operand(v1);
        }
        ValueCategory::Two => {
            // Form 2
            let v1 = frame.pop_operand();
            let v2 = pop_operand_if_category_matches!(frame, ValueCategory::Two);

            frame.push_operand(v1);
            frame.push_operand(v2);
            frame.push_operand(v1);
        }
    }
    Ok(())
}

fn instr_dup2_x2(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    match frame.peek_operand().category() {
        ValueCategory::One => {
            // Form 1, Form 3
            let v1 = frame.pop_operand();
            let v2 = pop_operand_if_category_matches!(frame, ValueCategory::One);
            match frame.peek_operand().category() {
                ValueCategory::One => {
                    // Form 1
                    let v3 = frame.pop_operand();
                    let v4 = pop_operand_if_category_matches!(frame, ValueCategory::One);

                    frame.push_operand(v2);
                    frame.push_operand(v1);
                    frame.push_operand(v4);
                    frame.push_operand(v3);
                    frame.push_operand(v2);
                    frame.push_operand(v1);
                }
                ValueCategory::Two => {
                    // Form 3
                    let v3 = frame.pop_operand();

                    frame.push_operand(v2);
                    frame.push_operand(v1);
                    frame.push_operand(v3);
                    frame.push_operand(v2);
                    frame.push_operand(v1);
                }
            }
        }
        ValueCategory::Two => {
            // Form 2, Form 4
            let v1 = frame.pop_operand();
            match frame.peek_operand().category() {
                ValueCategory::One => {
                    // Form 2
                    let v2 = frame.pop_operand();
                    let v3 = pop_operand_if_category_matches!(frame, ValueCategory::One);

                    frame.push_operand(v1);
                    frame.push_operand(v3);
                    frame.push_operand(v2);
                    frame.push_operand(v1);
                }
                ValueCategory::Two => {
                    // Form 4
                    let v2 = frame.pop_operand();

                    frame.push_operand(v1);
                    frame.push_operand(v2);
                    frame.push_operand(v1);
                }
            }
        }
    }
    Ok(())
}

fn instr_swap(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let v1 = pop_operand_if_category_matches!(frame, ValueCategory::One);
    let v2 = pop_operand_if_category_matches!(frame, ValueCategory::One);

    frame.push_operand(v1);
    frame.push_operand(v2);
    Ok(())
}

macro_rules! instr_unary_op {
    ($name:ident, $op:tt, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let $vtype(v) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            frame.push_operand($vtype($op v));
            Ok(())
        }
    };
}

macro_rules! instr_binary_op {
    ($name:ident, $op:tt, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let $vtype(rhs) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            let $vtype(lhs) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            frame.push_operand($vtype(lhs $op rhs));
            Ok(())
        }
    };
}

macro_rules! instr_shift_op {
    // shift with zero-extension
    ($name:ident, $op:tt, u, Value::Int) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let Value::Int(v2) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'int'").into());
            };
            let Value::Int(v1) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'int'").into());
            };
            let s = v2 & 0x1F; // take lowest 5 bits
            frame.push_operand(Value::Int(((v1 as u32) $op s) as i32));
            Ok(())
        }
    };
    ($name:ident, $op:tt, u, Value::Long) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let Value::Int(v2) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'int'").into());
            };
            let Value::Long(v1) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'long'").into());
            };
            let s = v2 & 0x3F; // take lowest 6 bits
            frame.push_operand(Value::Long(((v1 as u64) $op s) as i64));
            Ok(())
        }
    };
    // shift with sign-extension
    ($name:ident, $op:tt, Value::Int) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let Value::Int(v2) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'int'").into());
            };
            let Value::Int(v1) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'int'").into());
            };
            let s = v2 & 0x1F; // take lowest 5 bits
            frame.push_operand(Value::Int(v1 $op s));
            Ok(())
        }
    };
    ($name:ident, $op:tt, Value::Long) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let Value::Int(v2) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'int'").into());
            };
            let Value::Long(v1) = frame.pop_operand() else {
                return Err(concat!("target operand is not type 'long'").into());
            };
            let s = v2 & 0x3F; // take lowest 6 bits
            frame.push_operand(Value::Long(v1 $op s));
            Ok(())
        }
    };
}

instr_binary_op!(instr_iadd, +, Value::Int, "int");
instr_binary_op!(instr_ladd, +, Value::Long, "long");
instr_binary_op!(instr_fadd, +, Value::Float, "float");
instr_binary_op!(instr_dadd, +, Value::Double, "double");

instr_binary_op!(instr_isub, -, Value::Int, "int");
instr_binary_op!(instr_lsub, -, Value::Long, "long");
instr_binary_op!(instr_fsub, -, Value::Float, "float");
instr_binary_op!(instr_dsub, -, Value::Double, "double");

instr_binary_op!(instr_imul, *, Value::Int, "int");
instr_binary_op!(instr_lmul, *, Value::Long, "long");
instr_binary_op!(instr_fmul, *, Value::Float, "float");
instr_binary_op!(instr_dmul, *, Value::Double, "double");

instr_binary_op!(instr_idiv, /, Value::Int, "int");
instr_binary_op!(instr_ldiv, /, Value::Long, "long");
instr_binary_op!(instr_fdiv, /, Value::Float, "float");
instr_binary_op!(instr_ddiv, /, Value::Double, "double");

instr_binary_op!(instr_irem, %, Value::Int, "int");
instr_binary_op!(instr_lrem, %, Value::Long, "long");
instr_binary_op!(instr_frem, %, Value::Float, "float");
instr_binary_op!(instr_drem, %, Value::Double, "double");

instr_unary_op!(instr_ineg, -, Value::Int, "int");
instr_unary_op!(instr_lneg, -, Value::Long, "long");
instr_unary_op!(instr_fneg, -, Value::Float, "float");
instr_unary_op!(instr_dneg, -, Value::Double, "double");

instr_shift_op!(instr_ishl, <<, Value::Int);
instr_shift_op!(instr_lshl, <<, Value::Long);

instr_shift_op!(instr_ishr, >>, Value::Int);
instr_shift_op!(instr_lshr, >>, Value::Long);

instr_shift_op!(instr_iushr, >>, u, Value::Int);
instr_shift_op!(instr_lushr, >>, u, Value::Long);

instr_binary_op!(instr_iand, &, Value::Int, "int");
instr_binary_op!(instr_land, &, Value::Long, "long");

instr_binary_op!(instr_ior, |, Value::Int, "int");
instr_binary_op!(instr_lor, |, Value::Long, "long");

instr_binary_op!(instr_ixor, ^, Value::Int, "int");
instr_binary_op!(instr_lxor, ^, Value::Long, "long");

// increment the value of the local (specified by index) by delta
// operands: target local index, delta(signed int)
#[allow(overflowing_literals)]
fn instr_iinc(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    let idx = frame.next_param_u8() as usize;
    let delta = frame.next_param_u8() as i8 as i32;
    let Value::Int(v) = frame.get_local(idx) else {
        return Err("target local is not type 'int'".into());
    };
    frame.set_local(idx, Value::Int(v + delta));
    Ok(())
}

macro_rules! instr_conversion {
    ($name:ident, $from:path, trunc, $to_raw:ty) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let $from(v) = frame.pop_operand() else {
                return Err(concat!("target operand has invalid type").into());
            };
            frame.push_operand(Value::Int((v as $to_raw) as i32));
            Ok(())
        }
    };
    ($name:ident, $from:path, $to:path, $to_raw:ty) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let $from(v) = frame.pop_operand() else {
                return Err(concat!("target operand has invalid type").into());
            };
            frame.push_operand($to(v as $to_raw));
            Ok(())
        }
    };
}

instr_conversion!(instr_i2l, Value::Int, Value::Long, i64);
instr_conversion!(instr_i2f, Value::Int, Value::Float, f32);
instr_conversion!(instr_i2d, Value::Int, Value::Double, f64);

instr_conversion!(instr_l2i, Value::Long, Value::Int, i32);
instr_conversion!(instr_l2f, Value::Long, Value::Float, f32);
instr_conversion!(instr_l2d, Value::Long, Value::Double, f64);

instr_conversion!(instr_f2i, Value::Float, Value::Int, i32);
instr_conversion!(instr_f2l, Value::Float, Value::Long, i64);
instr_conversion!(instr_f2d, Value::Float, Value::Double, f64);

instr_conversion!(instr_d2i, Value::Double, Value::Int, i32);
instr_conversion!(instr_d2l, Value::Double, Value::Long, i64);
instr_conversion!(instr_d2f, Value::Double, Value::Float, f32);

instr_conversion!(instr_i2b, Value::Int, trunc, i8);
instr_conversion!(instr_i2c, Value::Int, trunc, u16);
instr_conversion!(instr_i2s, Value::Int, trunc, i16);

// compare the top (rhs) and the 2nd-top (lhs) values of the operand stack, assuming both are Long values.
// push the result of the comparison to the operand stack.
fn instr_lcmp(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    let Value::Long(rhs) = frame.pop_operand() else {
        return Err("target operand is not type 'long'".into());
    };
    let Value::Long(lhs) = frame.pop_operand() else {
        return Err("target operand is not type 'long'".into());
    };
    let cmp = match lhs.cmp(&rhs) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    frame.push_operand(Value::Int(cmp));
    Ok(())
}

// compare the top (rhs) and the 2nd-top (lhs) values of the operand stack, assuming both are floating-point type values.
// push the result of the comparison to the operand stack. if either of the values is NaN, push $if_nan.
macro_rules! instr_compare_floats {
    ($name:ident, $vtype:path, $vtype_name:expr, $if_nan:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();

            let $vtype(rhs) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            let $vtype(lhs) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            let cmp = match lhs.partial_cmp(&rhs) {
                Some(std::cmp::Ordering::Less) => -1,
                Some(std::cmp::Ordering::Equal) => 0,
                Some(std::cmp::Ordering::Greater) => 1,
                None => $if_nan,
            };
            frame.push_operand(Value::Int(cmp));
            Ok(())
        }
    };
}

instr_compare_floats!(instr_fcmpl, Value::Float, "float", -1);
instr_compare_floats!(instr_fcmpg, Value::Float, "float", 1);
instr_compare_floats!(instr_dcmpl, Value::Double, "double", -1);
instr_compare_floats!(instr_dcmpg, Value::Double, "double", 1);

// compare the top of operand stack (v) with 0.
// if v $cmp_op 0, move PC to: {current PC} + {delta}
// operands: delta of PC(signed int)
macro_rules! instr_if_cond {
    ($name:ident, $cmp_op:tt) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();

            let pc_delta = frame.next_param_u16() as i16 as i32;
            let Value::Int(v) = frame.pop_operand() else {
                return Err("target operand is not type 'int'".into());
            };
            if v $cmp_op 0 {
                let jmp_dest = (frame.get_pc() as i32 + pc_delta) as u32;
                frame.jump_pc(jmp_dest);
            }
            Ok(())
        }
    };
}

instr_if_cond!(instr_ifeq, ==);
instr_if_cond!(instr_ifne, !=);
instr_if_cond!(instr_iflt, <);
instr_if_cond!(instr_ifle, <=);
instr_if_cond!(instr_ifgt, >);
instr_if_cond!(instr_ifge, >=);

// compare the top (rhs) and the 2nd-top (lhs) values of the operand stack.
// if lhs $cmp_op rhs, move PC to: {current PC} + {delta}
// operands: delta of PC(signed int)
macro_rules! instr_cmp_cond {
    ($name:ident, $cmp_op:tt, $vtype:path, $vtype_name:expr) => {
        #[allow(overflowing_literals)]
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();

            let pc_delta = frame.next_param_u16() as i16 as i32;
            let $vtype(rhs) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            let $vtype(lhs) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };
            if lhs $cmp_op rhs {
                let jmp_dest = (frame.get_pc() as i32 + pc_delta) as u32;
                frame.jump_pc(jmp_dest);
            }
            Ok(())
        }
    };
}

instr_cmp_cond!(instr_if_icmpeq, ==, Value::Int, "int");
instr_cmp_cond!(instr_if_icmpne, !=, Value::Int, "int");
instr_cmp_cond!(instr_if_icmplt, < , Value::Int, "int");
instr_cmp_cond!(instr_if_icmple, <=, Value::Int, "int");
instr_cmp_cond!(instr_if_icmpgt, > , Value::Int, "int");
instr_cmp_cond!(instr_if_icmpge, >=, Value::Int, "int");
instr_cmp_cond!(instr_if_acmpeq, ==, Value::Reference, "reference");
instr_cmp_cond!(instr_if_acmpne, !=, Value::Reference, "reference");

// move PC to: {current PC} + {delta}
// operands: delta of PC(signed int)
fn instr_goto(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    let pc_delta = frame.next_param_u16() as i16 as i32;
    let jmp_dest = (frame.get_pc() as i32 + pc_delta) as u32;
    frame.jump_pc(jmp_dest);
    Ok(())
}

// push the "return address" (PC for next instruction) to the operand stack, then jump to {current PC} + {delta}
// operands: delta of PC(signed int)
fn instr_jsr(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    let pc_delta = frame.next_param_u16() as i16 as i32;
    let jmp_dest = (frame.get_pc() as i32 + pc_delta) as u32;
    frame.push_operand(Value::ReturnAddress(frame.get_pc() + 3)); // next instruction is 3 bytes ahead from jsr
    frame.jump_pc(jmp_dest);
    Ok(())
}

// jump to the "return address" stored in the specified local (by index)
fn instr_ret(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    let idx = frame.next_param_u8() as usize;
    let Value::ReturnAddress(pc) = frame.get_local(idx) else {
        return Err("target local is not type 'returnAddress'".into());
    };
    frame.jump_pc(pc);
    Ok(())
}

fn instr_tableswitch(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    frame.skip_code_padding(4);
    let default = frame.next_param_u32() as i32;
    let low = frame.next_param_u32() as i32;
    let high = frame.next_param_u32() as i32;

    let Value::Int(idx) = frame.pop_operand() else {
        return Err("target operand is not type 'int'".into());
    };
    let offset = if idx < low || idx > high {
        default
    } else {
        let i = idx - low;
        assert!(i >= 0);
        for _ in 0..i {
            frame.next_param_u32();
        }
        frame.next_param_u32() as i32
    };

    let jmp_dest = (frame.get_pc() as i32 + offset) as u32;
    frame.jump_pc(jmp_dest);
    Ok(())
}

fn instr_lookupswitch(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    frame.skip_code_padding(4);
    let default = frame.next_param_u32() as i32;
    let n_pairs = frame.next_param_u32() as usize;

    let Value::Int(key) = frame.pop_operand() else {
        return Err("target operand is not type 'int'".into());
    };
    let offset = {
        let mut i = 0;
        loop {
            i += 1;
            if i > n_pairs {
                break default;
            }

            let match_key = frame.next_param_u32() as i32;
            let offset = frame.next_param_u32() as i32;
            if key == match_key {
                break offset;
            }
        }
    };

    let jmp_dest = (frame.get_pc() as i32 + offset) as u32;
    frame.jump_pc(jmp_dest);
    Ok(())
}

// return from the method
macro_rules! instr_return {
    ($name:ident, void) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            t.pop_frame();
            Ok(())
        }
    };
    ($name:ident, $vtype:path, $vtype_name:expr) => {
        fn $name(t: &mut Thread, _: &mut MethodArea, _: &mut Heap) -> InstructionResult {
            let frame = t.current_frame();
            let ret @ $vtype(_) = frame.pop_operand() else {
                return Err(concat!("target operand is not type '", $vtype_name, "'").into());
            };

            t.pop_frame();
            t.current_frame().push_operand(ret);
            Ok(())
        }
    };
}

instr_return!(instr_ireturn, Value::Int, "int");
instr_return!(instr_lreturn, Value::Long, "long");
instr_return!(instr_freturn, Value::Float, "float");
instr_return!(instr_dreturn, Value::Double, "double");
instr_return!(instr_areturn, Value::Reference, "reference");
instr_return!(instr_return, void);

// get a value of a static field
fn instr_getstatic(
    t: &mut Thread,
    meth_area: &mut MethodArea,
    heap: &mut Heap,
) -> InstructionResult {
    let (cls_name, fld_name) = {
        let frame = t.current_frame();

        let idx = frame.next_param_u16();
        let CPInfo::Fieldref {
            class_name, name, ..
        } = frame.get_cp_info(idx)
        else {
            return Err("invalid fieldref")?;
        };
        (class_name.clone(), name.clone())
    };

    let cls = meth_area.resolve_class(&cls_name)?;
    cls.initialize(t, meth_area, heap)?;

    let frame = t.current_frame();
    let field = meth_area.resolve_static_field(&cls_name, &fld_name)?;
    frame.push_operand(field.get());
    Ok(())
}

// put a value to a static field
fn instr_putstatic(
    t: &mut Thread,
    meth_area: &mut MethodArea,
    heap: &mut Heap,
) -> InstructionResult {
    let (cls_name, fld_name) = {
        let frame = t.current_frame();

        let idx = frame.next_param_u16();
        let CPInfo::Fieldref {
            class_name, name, ..
        } = frame.get_cp_info(idx)
        else {
            return Err("invalid fieldref")?;
        };
        (class_name.clone(), name.clone())
    };

    let cls = meth_area.resolve_class(&cls_name)?;
    cls.initialize(t, meth_area, heap)?;

    let frame = t.current_frame();
    let field = meth_area.resolve_static_field(&cls_name, &fld_name)?;
    field.put(frame.pop_operand());
    Ok(())
}

// get a value of a class instance field
fn instr_getfield(t: &mut Thread, _: &mut MethodArea, heap: &mut Heap) -> InstructionResult {
    let fld_name = {
        let frame = t.current_frame();
        let idx = frame.next_param_u16();
        let CPInfo::Fieldref { name, .. } = frame.get_cp_info(idx) else {
            return Err("invalid fieldref")?;
        };
        name.clone()
    };

    let frame = t.current_frame();
    let Value::Reference(r) = frame.pop_operand() else {
        return Err("operand is not a reference value")?;
    };
    let Some(rv) = heap.get(r) else {
        return Err("referent not found on heap")?;
    };
    let RefValue::Object(obj) = rv else {
        return Err("referent is not a object")?;
    };
    let Some(field) = obj.get_field(&fld_name) else {
        return Err(format!("field {fld_name} not found"))?;
    };

    frame.push_operand(field.get());
    Ok(())
}

// put a value to a class instance field
fn instr_putfield(t: &mut Thread, _: &mut MethodArea, heap: &mut Heap) -> InstructionResult {
    let fld_name = {
        let frame = t.current_frame();
        let idx = frame.next_param_u16();
        let CPInfo::Fieldref { name, .. } = frame.get_cp_info(idx) else {
            return Err("invalid fieldref")?;
        };
        name.clone()
    };

    let frame = t.current_frame();
    let Value::Reference(r) = frame.pop_operand() else {
        return Err("operand is not a reference value")?;
    };
    let Some(rv) = heap.get(r) else {
        return Err("referent not found on heap")?;
    };
    let RefValue::Object(obj) = rv else {
        return Err("referent is not a object")?;
    };
    let Some(field) = obj.get_field(&fld_name) else {
        return Err(format!("field {fld_name} not found"))?;
    };

    field.put(frame.pop_operand());
    Ok(())
}

fn instr_invokevirtual(
    t: &mut Thread,
    meth_area: &mut MethodArea,
    heap: &mut Heap,
) -> InstructionResult {
    let (_cls_name, meth_name, desc) = {
        let frame = t.current_frame();
        let idx = frame.next_param_u16();
        let CPInfo::Methodref {
            class_name,
            name,
            descriptor,
        } = frame.get_cp_info(idx)
        else {
            return Err("invalid methodref")?;
        };
        (class_name.clone(), name.clone(), descriptor.clone())
    };
    // resolve class referenced by method ref
    let _ = meth_area.resolve_class(&ref_cls_name)?;

    // get receiver object
    let frame = t.current_frame();
    let this @ Value::Reference(r) = frame.pop_operand() else {
        return Err("operand is not a reference value")?;
    };
    let Some(rv) = heap.get(r) else {
        return Err("referent not found on heap")?;
    };
    let RefValue::Object(obj) = rv else {
        return Err("referent is not a object")?;
    };

    // lookup method to be called
    let rt_cls = obj.get_class();
    let sig = MethodSignature::new(&meth_name, &desc);
    let meth = meth_area.lookup_instance_method("TODO", &rt_cls, &sig)?;
    let num_args = meth.num_args();

    // method call
    // create new frame for the method, transfer the receiver(`this`) and args to the frame, then push onto frame stack
    let caller_frame = t.current_frame();
    let mut callee_frame = Frame::new(rt_cls, meth);
    Frame::transfer_receiver_and_args(caller_frame, &mut callee_frame, this, num_args);
    t.push_frame(callee_frame);

    Ok(())
}

fn instr_invokespecial(
    t: &mut Thread,
    meth_area: &mut MethodArea,
    _: &mut Heap,
) -> InstructionResult {
    let (cls_name, meth_name, desc) = {
        let frame = t.current_frame();
        let idx = frame.next_param_u16();
        let CPInfo::Methodref {
            class_name,
            name,
            descriptor,
        } = frame.get_cp_info(idx)
        else {
            return Err("invalid methodref")?;
        };
        (class_name.clone(), name.clone(), descriptor.clone())
    };
    if meth_name != "<init>" {
        Err("invoking methods other than <init> is currently not supported")?;
    }

    let frame = t.current_frame();
    let this @ Value::Reference(_) = frame.pop_operand() else {
        return Err("operand is not a reference value")?;
    };

    // lookup method to be called
    let cls = meth_area.resolve_class(&cls_name)?;
    let sig = MethodSignature::new(&meth_name, &desc);
    let meth = cls
        .lookup_instance_method(&sig)
        .ok_or("instance method {cls.name}.{sig} not found")?;
    let num_args = meth.num_args();

    // method call
    // create new frame for the method, transfer the receiver(`this`) and args to the frame, then push onto frame stack
    let caller_frame = t.current_frame();
    let mut callee_frame = Frame::new(cls, meth);
    Frame::transfer_receiver_and_args(caller_frame, &mut callee_frame, this, num_args);
    t.push_frame(callee_frame);

    Ok(())
}

fn instr_invokestatic(
    t: &mut Thread,
    meth_area: &mut MethodArea,
    heap: &mut Heap,
) -> InstructionResult {
    let (cls_name, meth_name, desc) = {
        let frame = t.current_frame();

        // lookup methodref from const pool
        let idx = frame.next_param_u16();
        let CPInfo::Methodref {
            class_name,
            name,
            descriptor,
        } = frame.get_cp_info(idx)
        else {
            return Err("invalid methodref")?;
        };
        (class_name.clone(), name.clone(), descriptor.clone())
    };

    let cls = meth_area.resolve_class(&cls_name)?;
    cls.initialize(t, meth_area, heap)?;

    // lookup method to be called
    let sig = MethodSignature::new(&meth_name, &desc);
    let (cls, meth) = meth_area.lookup_static_method(&cls_name, &sig)?;
    let num_args = meth.num_args();

    // method call
    // create new frame for the method, transfer args to the frame, then push onto frame stack
    let caller_frame = t.current_frame();
    let mut callee_frame = Frame::new(cls, meth);
    Frame::transfer_args(caller_frame, &mut callee_frame, num_args);
    t.push_frame(callee_frame);

    Ok(())
}

fn instr_new(t: &mut Thread, meth_area: &mut MethodArea, heap: &mut Heap) -> InstructionResult {
    let cls_name = {
        let frame = t.current_frame();
        let idx = frame.next_param_u16();
        let CPInfo::Class { name } = frame.get_cp_info(idx) else {
            return Err("not class")?;
        };
        name
    };

    let cls = meth_area.resolve_class(cls_name)?;
    cls.clone().initialize(t, meth_area, heap)?;

    let rv = heap.alloc_object(cls.clone());
    t.current_frame().push_operand(rv);

    Ok(())
}

fn instr_newarray(t: &mut Thread, _: &mut MethodArea, heap: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();

    let Value::Int(length) = frame.pop_operand() else {
        return Err("invalid type for length of array")?;
    };

    // cf. Table 6.5.newarray-A
    let atype = frame.next_param_u8();
    let item_desc = match atype {
        4 => "Z",  // boolean
        5 => "C",  // char
        6 => "F",  // float
        7 => "D",  // double
        8 => "B",  // byte
        9 => "S",  // short
        10 => "I", // int
        11 => "J", // long
        _ => unreachable!(),
    };

    let rv = heap.alloc_array(length as usize, item_desc);
    t.current_frame().push_operand(rv);

    Ok(())
}

fn instr_anewarray(
    t: &mut Thread,
    meth_area: &mut MethodArea,
    heap: &mut Heap,
) -> InstructionResult {
    let (length, cls_name) = {
        let frame = t.current_frame();
        let Value::Int(length) = frame.pop_operand() else {
            return Err("invalid type for length of array")?;
        };

        let idx = frame.next_param_u16();
        let CPInfo::Class { name } = frame.get_cp_info(idx) else {
            return Err("not class")?;
        };
        (length, name.clone())
    };

    // resolve the "innermost" class of array element
    let innermost_cls_name = cls_name.trim_start_matches('[');
    _ = meth_area.resolve_class(innermost_cls_name);

    let is_array = cls_name.starts_with("[");
    let item_desc = if is_array {
        cls_name
    } else {
        format!("L{cls_name};")
    };

    let rv = heap.alloc_array(length as usize, &item_desc);
    t.current_frame().push_operand(rv);

    Ok(())
}

// get the length of an array
fn instr_arraylength(t: &mut Thread, _: &mut MethodArea, heap: &mut Heap) -> InstructionResult {
    let frame = t.current_frame();
    let Value::Reference(r) = frame.pop_operand() else {
        return Err("operand is not a reference value")?;
    };
    let Some(rv) = heap.get(r) else {
        return Err("referent not found on heap")?;
    };
    let RefValue::Array(arr) = rv else {
        return Err("referent is not an array")?;
    };

    frame.push_operand(Value::Int(arr.get_length() as i32));
    Ok(())
}
