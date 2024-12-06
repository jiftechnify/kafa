use std::cell::Cell;

use crate::class_file::CPInfo;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Char(u16),
    Float(f32),
    Double(f64),
    Reference(usize),
    ReturnAddress(u32),
}

pub enum ValueCategory {
    One,
    Two,
}

impl Value {
    pub fn category(&self) -> ValueCategory {
        match &self {
            Value::Long(_) | Value::Double(_) => ValueCategory::Two,
            _ => ValueCategory::One,
        }
    }
}

#[derive(Clone)]
pub struct MutValue(Cell<Value>);

impl MutValue {
    pub fn from_val(val: Value) -> Self {
        MutValue(Cell::new(val))
    }

    pub fn from_cp_info(cp_info: &CPInfo) -> Self {
        use CPInfo::*;
        let val = match cp_info {
            Integer(i) => Value::Int(*i),
            Float(f) => Value::Float(*f),
            Long(l) => Value::Long(*l),
            Double(d) => Value::Double(*d),
            String { .. } => {
                eprintln!("string constant is not supported");
                Value::Reference(0) // null
            }
            _ => {
                eprintln!("not a constant value");
                Value::Reference(0) // null
            }
        };
        MutValue::from_val(val)
    }

    pub fn default_of_type(desc: &str) -> Self {
        assert!(!desc.is_empty());
        let Some(fst_char) = desc.chars().nth(0) else {
            unreachable!()
        };
        let default_val = match fst_char {
            'B' => Value::Byte(0),
            'C' => Value::Char(0),
            'D' => Value::Double(0.0),
            'F' => Value::Float(0.0),
            'I' => Value::Int(0),
            'J' => Value::Long(0),
            'L' => Value::Reference(0), // null
            'S' => Value::Short(0),
            // 'Z' -> boolean
            // the Java programming language that operate on boolean values
            // are compiled to use values of the Java Virtual Machine int data type.
            'Z' => Value::Int(0),
            '[' => Value::Reference(0), // null reference to arrays
            _ => unreachable!(),
        };
        MutValue::from_val(default_val)
    }
}

impl MutValue {
    pub fn get(&self) -> Value {
        self.0.get()
    }

    pub fn put(&self, new_val: Value) {
        self.0.set(new_val);
    }
}
