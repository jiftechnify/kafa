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
    Class {
        name_idx: u16,
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
        // CONSTANT_Class
        7 => CPInfo::Class {
            name_idx: bs.read_u16(),
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
