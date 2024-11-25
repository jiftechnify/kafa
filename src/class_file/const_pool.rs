use crate::support::ByteSeq;

#[derive(Debug)]
pub struct ConstantPool(Vec<CPInfo>);

impl ConstantPool {
    pub fn parse(bs: &mut ByteSeq) -> Result<ConstantPool, Box<dyn std::error::Error + 'static>> {
        let count = bs.read_u16() as usize;
        let mut cp = Vec::with_capacity(count - 1);
        for _ in 0..count - 1 {
            cp.push(parse_cp_info(bs)?);
        }
        Ok(ConstantPool(cp))
    }

    pub fn get_utf8(&self, idx: u16) -> &str {
        if let CPInfo::Utf8(s) = self.get_info(idx) {
            s.as_str()
        } else {
            eprintln!("not a CONSTANT_Utf8");
            ""
        }
    }

    fn get_info(&self, idx: u16) -> &CPInfo {
        assert!(idx > 0);
        &self.0[idx as usize - 1]
    }
}

/// Class/MethodRef/NameAndTypeが持つインデックスは、ConstantPool内の他のエントリへの参照。
/// インデックスは1-オリジンであることに注意!
#[derive(Debug)]
enum CPInfo {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name_idx: u16,
    },
    String {
        string_idx: u16,
    },
    Methodref {
        class_idx: u16,
        name_and_type_idx: u16,
    },
    NameAndType {
        name_idx: u16,
        descriptor_idx: u16, // format: (<param type>*)<return type>
    },
    Unsupported,
}

fn parse_cp_info(bs: &mut ByteSeq) -> Result<CPInfo, Box<dyn std::error::Error + 'static>> {
    let tag = bs.read_u8();
    let parsed = match tag {
        // CONSTANT_Utf8
        1 => {
            let len = bs.read_u16() as usize;
            let s = bs.read_bytes(len);
            CPInfo::Utf8(String::from_utf8(s)?)
        }
        // CONSTANT_Integer
        3 => {
            let n = bs.read_u32();
            CPInfo::Integer(n as i32)
        }
        // CONSTANT_Float
        4 => {
            let n = bs.read_u32();
            CPInfo::Float(f32::from_bits(n))
        }
        // CONSTANT_Long
        5 => {
            let n = bs.read_u64();
            CPInfo::Long(n as i64)
        }
        // CONSTANT_Double
        6 => {
            let n = bs.read_u64();
            CPInfo::Double(f64::from_bits(n))
        }
        // CONSTANT_Class
        7 => CPInfo::Class {
            name_idx: bs.read_u16(),
        },
        // CONSTANT_String
        8 => CPInfo::String {
            string_idx: bs.read_u16(),
        },
        // CONSTANT_Methodref
        10 => CPInfo::Methodref {
            class_idx: bs.read_u16(),
            name_and_type_idx: bs.read_u16(),
        },
        // CONSTANT_NameAndType
        12 => CPInfo::NameAndType {
            name_idx: bs.read_u16(),
            descriptor_idx: bs.read_u16(),
        },
        _ => {
            eprintln!("skipping unsupported constant pool info type: {}", tag);
            CPInfo::Unsupported
        }
    };
    Ok(parsed)
}
