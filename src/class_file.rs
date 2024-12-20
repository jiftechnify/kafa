use std::{fmt::Display, io::Read};

use crate::support::ByteSeq;

mod attr;
mod const_pool;

use attr::{parse_attributes, Attribute, CodeAttr};
use bitflags::bitflags;
pub use const_pool::{CPInfo, ConstantPool};

bitflags! {
    #[derive(Clone, Debug)]
    pub struct ClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
}

impl ClassAccessFlags {
    pub fn is_interface(&self) -> bool {
        self.contains(ClassAccessFlags::INTERFACE)
    }
}

#[derive(Debug)]
pub struct ClassFile {
    pub constant_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
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
        let access_flags = ClassAccessFlags::from_bits_retain(bs.read_u16());

        // parse this_class and super_class
        let this_class = parse_class_ref(&mut bs, &cp).unwrap();
        let super_class = parse_class_ref(&mut bs, &cp);
        // parse interfaces
        let ifaces_count = bs.read_u16() as usize;
        let mut interfaces = Vec::with_capacity(ifaces_count);
        interfaces.resize_with(ifaces_count, || parse_class_ref(&mut bs, &cp).unwrap());

        // parse fields and methods
        let fields = parse_fields(&mut bs, &cp);
        let methods = parse_methods(&mut bs, &cp);

        // skip attributes

        Ok(ClassFile {
            constant_pool: cp,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
        })
    }
}

fn parse_class_ref(bs: &mut ByteSeq, cp: &ConstantPool) -> Option<String> {
    let idx = bs.read_u16();
    match idx {
        0 => None,
        _ => {
            let c = cp.get_class(idx);
            Some(c.name.to_string())
        }
    }
}

bitflags! {
    #[derive(Clone, Debug)]
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

impl FieldAccessFlags {
    pub fn is_static(&self) -> bool {
        self.contains(FieldAccessFlags::STATIC)
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
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
    #[derive(Clone, Debug)]
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

impl MethodAccessFlags {
    pub fn is_static(&self) -> bool {
        self.contains(MethodAccessFlags::STATIC) && !self.contains(MethodAccessFlags::ABSTRACT)
    }

    /// whether the method should have Code attribute? == neither native nor abstract
    pub fn should_have_code(&self) -> bool {
        !self.intersects(MethodAccessFlags::NATIVE | MethodAccessFlags::ABSTRACT)
    }

    pub fn is_public_non_static(&self) -> bool {
        self.contains(MethodAccessFlags::PUBLIC) && !self.contains(MethodAccessFlags::STATIC)
    }

    /// whether the method is `default` interface method?
    pub fn is_interface_default(&self) -> bool {
        !self.intersects(
            MethodAccessFlags::PRIVATE | MethodAccessFlags::STATIC | MethodAccessFlags::ABSTRACT,
        )
    }
}

#[cfg(test)]
mod test_meth_access_flags {
    use super::MethodAccessFlags;

    #[test]
    fn test_is_static() {
        let tests = [
            (MethodAccessFlags::empty(), false),
            (MethodAccessFlags::STATIC, true),
            (MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC, true),
            (MethodAccessFlags::FINAL, false),
            (
                MethodAccessFlags::STATIC | MethodAccessFlags::ABSTRACT,
                false,
            ),
            (MethodAccessFlags::all() - MethodAccessFlags::ABSTRACT, true),
            (MethodAccessFlags::all(), false),
        ];
        for (flags, exp) in tests {
            assert_eq!(flags.is_static(), exp);
        }
    }

    #[test]
    fn test_should_have_code() {
        let tests = [
            (MethodAccessFlags::empty(), true),
            (MethodAccessFlags::PUBLIC, true),
            (MethodAccessFlags::ABSTRACT, false),
            (MethodAccessFlags::NATIVE, false),
            (
                MethodAccessFlags::all() - MethodAccessFlags::ABSTRACT - MethodAccessFlags::NATIVE,
                true,
            ),
            (MethodAccessFlags::all(), false),
        ];
        for (flags, exp) in tests {
            assert_eq!(flags.should_have_code(), exp);
        }
    }
}

#[derive(Debug)]
pub struct MethodInfo {
    access_flags: MethodAccessFlags,
    name: String,
    descriptor: String,
    attributes: Vec<Attribute>,
}

pub struct MethodComponents {
    pub access_flags: MethodAccessFlags,
    pub name: String,
    pub descriptor: String,

    pub code_attr: Option<CodeAttr>,
}

impl MethodInfo {
    pub fn into_components(self) -> MethodComponents {
        for attr in self.attributes.into_iter() {
            if let Attribute::Code(c) = attr {
                return MethodComponents {
                    access_flags: self.access_flags,
                    name: self.name,
                    descriptor: self.descriptor,
                    code_attr: Some(c),
                };
            }
        }

        if self.access_flags.should_have_code() {
            unreachable!("MethodInfo must have Code attribute!");
        } else {
            MethodComponents {
                access_flags: self.access_flags,
                name: self.name,
                descriptor: self.descriptor,
                code_attr: None,
            }
        }
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
