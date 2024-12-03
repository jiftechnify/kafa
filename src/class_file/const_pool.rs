use crate::support::ByteSeq;

#[derive(Debug, Clone)]
pub struct ConstantPool(pub Vec<CPInfo>);

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

    pub fn get_info(&self, idx: u16) -> &CPInfo {
        assert!(0 < idx && idx <= self.0.len() as u16);
        &self.0[idx as usize - 1]
    }

    pub(in crate::class_file) fn get_class(&self, idx: u16) -> CPClassResolved {
        let CPInfo::Class { name_idx } = self.get_info(idx) else {
            eprintln!("not a CONSTANT_Class");
            return CPClassResolved::default();
        };
        CPClassResolved {
            name: self.get_utf8(*name_idx),
        }
    }

    pub(in crate::class_file) fn get_name_and_type(&self, idx: u16) -> CPNameAndTypeResolved {
        let CPInfo::NameAndType {
            name_idx,
            descriptor_idx,
        } = self.get_info(idx)
        else {
            eprintln!("not a CONSTANT_NameAndType");
            return CPNameAndTypeResolved::default();
        };
        CPNameAndTypeResolved {
            name: self.get_utf8(*name_idx),
            descriptor: self.get_utf8(*descriptor_idx),
        }
    }

    pub(in crate::class_file) fn get_method_ref(&self, idx: u16) -> CPMethodrefResolved {
        let CPInfo::Methodref {
            class_idx,
            name_and_type_idx,
        } = self.get_info(idx)
        else {
            eprintln!("not a CONSTANT_Methodref");
            return CPMethodrefResolved::default();
        };
        let cls = self.get_class(*class_idx);
        let nt = self.get_name_and_type(*name_and_type_idx);
        CPMethodrefResolved {
            class: cls,
            name_and_type: nt,
        }
    }

    pub fn infos(&self) -> impl Iterator<Item = &CPInfo> {
        self.0.iter()
    }
}

#[derive(Default)]
pub(in crate::class_file) struct CPClassResolved<'a> {
    pub name: &'a str,
}

#[derive(Default)]
pub(in crate::class_file) struct CPNameAndTypeResolved<'a> {
    name: &'a str,
    descriptor: &'a str,
}

#[derive(Default)]
pub(in crate::class_file) struct CPFieldrefResolved<'a> {
    class: CPClassResolved<'a>,
    name_and_type: CPNameAndTypeResolved<'a>,
}

#[derive(Default)]
pub(in crate::class_file) struct CPMethodrefResolved<'a> {
    class: CPClassResolved<'a>,
    name_and_type: CPNameAndTypeResolved<'a>,
}

/// Class/MethodRef/NameAndTypeが持つインデックスは、ConstantPool内の他のエントリへの参照。
/// インデックスは1-オリジンであることに注意!
#[derive(Debug, Clone)]
pub enum CPInfo {
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
    Fieldref {
        class_idx: u16,
        name_and_type_idx: u16,
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
        // CONSTANT_Fieldref
        9 => CPInfo::Fieldref {
            class_idx: bs.read_u16(),
            name_and_type_idx: bs.read_u16(),
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
        // skip unsupported cp info type
        16 | 19 | 20 => skip_unsupported_cp_info(bs, tag, 2),
        15 => skip_unsupported_cp_info(bs, tag, 3),
        11 | 17 | 18 => skip_unsupported_cp_info(bs, tag, 4),
        _ => {
            eprintln!("invalid cp info tag");
            unreachable!()
        }
    };
    Ok(parsed)
}

fn skip_unsupported_cp_info(bs: &mut ByteSeq, tag: u8, n: usize) -> CPInfo {
    eprintln!("skipping unsupported constant pool info type: {}", tag);
    bs.skip(n);
    CPInfo::Unsupported
}
