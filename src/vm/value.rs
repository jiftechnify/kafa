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
    ReturnAddress(usize),
}

pub enum ValueType {
    Byte,
    Short,
    Int,
    Long,
    Char,
    Float,
    Double,
    Reference,
    ReturnAddress,
}
