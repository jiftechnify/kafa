use std::{fmt::Display, io::Read};

use crate::support::ByteSeq;

mod attr;
mod const_pool;

use attr::{parse_attributes, Attribute};
use bitflags::bitflags;
pub use const_pool::{CPInfo, ConstantPool};

#[derive(Debug)]
pub struct ClassFile {
    pub constant_pool: ConstantPool,
    pub this_class: String,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
}

impl ClassFile {
    const MAGIC_NUMBER: u32 = 0xCAFEBABE;

    pub fn parse<R: Read>(input: R) -> Result<ClassFile, Box<dyn std::error::Error + 'static>> {
        let mut bs = ByteSeq::new(input)?;

        // Check if it starts with magic number
        if bs.read_u32() != Self::MAGIC_NUMBER {
            Err("not a java class file")?;
        }
        // skip major and minor version
        bs.skip(4);

        // parse constant pool
        let cp = ConstantPool::parse(&mut bs)?;

        // skip access_flags
        bs.skip(2);
        // parse this_class
        let this_class = parse_class_ref(&mut bs, &cp);
        // skip super_class
        bs.skip(2);
        // skip interfaces
        let ifaces_count = bs.read_u16() as usize;
        bs.skip(2 * ifaces_count);

        // parse fields and methods
        let fields = parse_fields(&mut bs, &cp);
        let methods = parse_methods(&mut bs, &cp);

        // skip attributes

        Ok(ClassFile {
            constant_pool: cp,
            this_class,
            fields,
            methods,
        })
    }
}

fn parse_class_ref(bs: &mut ByteSeq, cp: &ConstantPool) -> String {
    let idx = bs.read_u16();
    let c = cp.get_class(idx);
    c.name.to_string()
}

bitflags! {
    #[derive(Debug)]
    pub struct FieldAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    access_flags: FieldAccessFlags,
    pub name: String,
    pub descriptor: String,
    attributes: Vec<Attribute>,
}

impl FieldInfo {
    pub fn get_const_val(&self) -> Option<&CPInfo> {
        for attr in self.attributes.iter() {
            if let Attribute::ConstantValue(const_val_attr) = attr {
                return Some(&const_val_attr.const_value);
            }
        }
        None
    }

    pub fn is_static(&self) -> bool {
        self.access_flags.contains(FieldAccessFlags::STATIC)
    }
}

fn parse_fields(bs: &mut ByteSeq, cp: &ConstantPool) -> Vec<FieldInfo> {
    let count = bs.read_u16() as usize;
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        vec.push(parse_field_info(bs, cp));
    }
    vec
}

fn parse_field_info(bs: &mut ByteSeq, cp: &ConstantPool) -> FieldInfo {
    FieldInfo {
        access_flags: FieldAccessFlags::from_bits_retain(bs.read_u16()),
        name: cp.get_utf8(bs.read_u16()).to_string(),
        descriptor: cp.get_utf8(bs.read_u16()).to_string(),
        attributes: parse_attributes(bs, cp),
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct MethodAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT =  0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
}

#[derive(Debug)]
pub struct MethodInfo {
    access_flags: MethodAccessFlags,
    name: String,
    descriptor: String,
    attributes: Vec<Attribute>,
}

impl MethodInfo {
    // TODO: refactor this
    pub fn into_components(self) -> (String, String, u16, u16, Vec<u8>) {
        for attr in self.attributes.into_iter() {
            if let Attribute::Code(c) = attr {
                return (
                    self.name,
                    self.descriptor,
                    c.max_stack,
                    c.max_locals,
                    c.code,
                );
            }
        }
        eprintln!("Code attr not found");
        (String::new(), String::new(), 0, 0, Vec::new())
    }

    pub fn is_static(&self) -> bool {
        self.access_flags.contains(MethodAccessFlags::STATIC)
            && !self.access_flags.contains(MethodAccessFlags::ABSTRACT)
    }
}

impl Display for MethodInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.descriptor)
    }
}

fn parse_methods(bs: &mut ByteSeq, cp: &ConstantPool) -> Vec<MethodInfo> {
    let count = bs.read_u16() as usize;
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        vec.push(parse_method_info(bs, cp));
    }
    vec
}

fn parse_method_info(bs: &mut ByteSeq, cp: &ConstantPool) -> MethodInfo {
    MethodInfo {
        access_flags: MethodAccessFlags::from_bits_retain(bs.read_u16()),
        name: cp.get_utf8(bs.read_u16()).to_string(),
        descriptor: cp.get_utf8(bs.read_u16()).to_string(),
        attributes: parse_attributes(bs, cp),
    }
}
