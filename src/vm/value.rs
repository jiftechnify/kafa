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
