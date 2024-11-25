use std::{fmt::Display, io::Read};

use crate::support::ByteSeq;

mod attr;
mod const_pool;

use attr::{parse_attributes, Attribute, CodeAttr};
pub use const_pool::{CPInfo, ConstantPool};

#[derive(Debug)]
pub struct ClassFile {
    pub constant_pool: ConstantPool,
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

        // skip access_flags, this_class, super_class
        bs.skip(6);
        // skip interfaces
        let ifaces_count = bs.read_u16() as usize;
        bs.skip(2 * ifaces_count);

        // skip fields
        let _ = parse_fields(&mut bs, &cp);

        // parse methods
        let methods = parse_methods(&mut bs, &cp);

        // skip attributes

        Ok(ClassFile {
            constant_pool: cp,
            methods,
        })
    }
}

impl ClassFile {
    // lookup the method by the name and the signature(descriptor)
    pub fn find_method(&self, name: &str, desc: &str) -> Option<&MethodInfo> {
        self.methods
            .iter()
            .find(|m| m.name == name && m.descriptor == desc)
    }
}

#[derive(Debug)]
struct FieldInfo {
    access_flags: u16,
    name: String,
    descriptor: String,
    attributes: Vec<Attribute>,
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
        access_flags: bs.read_u16(),
        name: cp.get_utf8(bs.read_u16()).to_string(),
        descriptor: cp.get_utf8(bs.read_u16()).to_string(),
        attributes: parse_attributes(bs, cp),
    }
}

#[derive(Debug)]
pub struct MethodInfo {
    access_flags: u16,
    name: String,
    descriptor: String,
    attributes: Vec<Attribute>,
}

impl MethodInfo {
    pub fn get_code_attr(&self) -> Option<&CodeAttr> {
        for attr in self.attributes.iter() {
            if let Attribute::Code(code_attr) = attr {
                return Some(code_attr);
            }
        }
        None
    }

    pub fn num_args(&self) -> usize {
        self.descriptor[1..].find(')').expect("malformed signature")
    }
}

impl Display for MethodInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name, self.descriptor)
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
        access_flags: bs.read_u16(),
        name: cp.get_utf8(bs.read_u16()).to_string(),
        descriptor: cp.get_utf8(bs.read_u16()).to_string(),
        attributes: parse_attributes(bs, cp),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_method_info_num_args() {
        let tests = vec![("()V", 0), ("(I)V", 1), ("(ISB)V", 3)];

        for (input, exp) in tests {
            let m = MethodInfo {
                access_flags: 0,
                name: "".to_string(),
                descriptor: input.to_string(),
                attributes: vec![],
            };
            assert_eq!(m.num_args(), exp);
        }
    }
}
