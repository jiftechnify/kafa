use crate::class_file::const_pool::ConstantPool;
use crate::support::ByteSeq;

#[derive(Debug)]
pub enum Attribute {
    Code(CodeAttr),
    Unsupported,
}

pub fn parse_attributes(bs: &mut ByteSeq, cp: &ConstantPool) -> Vec<Attribute> {
    let count = bs.read_u16() as usize;
    let mut vec = Vec::with_capacity(count);
    for _ in 0..count {
        let name_idx = bs.read_u16();
        let name = cp.get_utf8(name_idx);
        let len = bs.read_u32() as usize;

        let attr = match name {
            // Code_attribute
            CodeAttr::NAME => {
                let code_attr = parse_code_attr(bs, cp);
                Attribute::Code(code_attr)
            }
            _ => {
                eprintln!("skipping unsupported attribute: {}", name);
                bs.skip(len);
                Attribute::Unsupported
            }
        };
        vec.push(attr);
    }
    vec
}

#[derive(Debug)]
pub struct CodeAttr {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
}

impl CodeAttr {
    const NAME: &str = "Code";
}

fn parse_code_attr(bs: &mut ByteSeq, cp: &ConstantPool) -> CodeAttr {
    let max_stack = bs.read_u16();
    let max_locals = bs.read_u16();

    let code_len = bs.read_u32() as usize;
    let code = bs.read_bytes(code_len);

    // skip exception table(8 * len)
    let exc_tbl_len = bs.read_u16() as usize;
    bs.skip(8 * exc_tbl_len);

    // skip attributes
    let _ = parse_attributes(bs, cp);

    CodeAttr {
        max_stack,
        max_locals,
        code,
    }
}
