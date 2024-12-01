use std::collections::HashMap;

use crate::class_file::{CPInfo, ConstantPool};

pub struct Class {
    pub name: String,
    const_pool: RunTimeConstantPool,
    static_methods: HashMap<MethodSignature, Method>,
}

impl Class {
    pub fn from_class_file(cls_file: crate::class_file::ClassFile) -> Class {
        let rtcp = RunTimeConstantPool::from_class_file_cp(cls_file.constant_pool);

        let mut static_methods = HashMap::new();
        for m in cls_file.methods.into_iter().filter(|m| m.is_static()) {
            let (name, desc, max_stack, max_locals, code) = m.into_fields();
            let sig = MethodSignature {
                name,
                descriptor: MethodDescriptor(desc),
            };
            let method = Method {
                signature: sig.clone(),
                max_stack,
                max_locals,
                code,
            };
            static_methods.insert(sig, method);
        }

        Class {
            name: cls_file.this_class,
            const_pool: rtcp,
            static_methods,
        }
    }

    pub fn dummy() -> Class {
        Class {
            name: "dummy".to_string(),
            const_pool: RunTimeConstantPool::empty(),
            static_methods: HashMap::new(),
        }
    }
}

impl Class {
    pub fn lookup_static_method(&self, signature: &MethodSignature) -> Option<Method> {
        self.static_methods.get(signature).cloned()
    }

    pub fn get_cp_info(&self, idx: u16) -> &RunTimeCPInfo {
        self.const_pool.get_info(idx)
    }
}

pub struct FieldDescriptor(String);

impl std::fmt::Display for FieldDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MethodDescriptor(String);

impl MethodDescriptor {
    fn num_args(&self) -> usize {
        self.0[1..].find(')').expect("malformed signature")
    }
}

impl std::fmt::Display for MethodDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MethodSignature {
    pub name: String,
    pub descriptor: MethodDescriptor,
}

impl MethodSignature {
    pub fn new(name: &str, desc: &str) -> MethodSignature {
        MethodSignature {
            name: name.to_string(),
            descriptor: MethodDescriptor(desc.to_string()),
        }
    }
}

impl std::fmt::Display for MethodSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name, self.descriptor)
    }
}
impl Default for MethodSignature {
    fn default() -> Self {
        Self {
            name: String::new(),
            descriptor: MethodDescriptor("()V".to_string()),
        }
    }
}

#[derive(Clone)]
pub struct Method {
    pub signature: MethodSignature,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
}
impl Method {
    pub fn num_args(&self) -> usize {
        self.signature.descriptor.num_args()
    }
}

pub struct RunTimeConstantPool(Vec<RunTimeCPInfo>);
pub enum RunTimeCPInfo {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name: String,
    },
    String(String),
    Fieldref {
        class_name: String,
        name: String,
        descriptor: String,
    },
    Methodref {
        class_name: String,
        name: String,
        descriptor: String,
    },
    NameAndType {
        name: String,
        descriptor: String, // format: (<param type>*)<return type>
    },
    Unsupported,
}

impl RunTimeConstantPool {
    pub fn from_class_file_cp(cp: crate::class_file::ConstantPool) -> RunTimeConstantPool {
        use RunTimeCPInfo::*;

        let resolved = cp
            .infos()
            .map(|cp_info| match cp_info {
                CPInfo::Utf8(s) => Utf8(s.clone()),
                CPInfo::Integer(i) => Integer(*i),
                CPInfo::Float(f) => Float(*f),
                CPInfo::Long(l) => Long(*l),
                CPInfo::Double(d) => Double(*d),
                CPInfo::Class { name_idx } => Class {
                    name: cp.get_utf8(*name_idx).to_string(),
                },
                CPInfo::String { string_idx } => String(cp.get_utf8(*string_idx).to_string()),
                CPInfo::Fieldref {
                    class_idx,
                    name_and_type_idx,
                } => resolve_fieldref(&cp, *class_idx, *name_and_type_idx),
                CPInfo::Methodref {
                    class_idx,
                    name_and_type_idx,
                } => resolve_methodref(&cp, *class_idx, *name_and_type_idx),
                CPInfo::NameAndType {
                    name_idx,
                    descriptor_idx,
                } => NameAndType {
                    name: cp.get_utf8(*name_idx).to_string(),
                    descriptor: cp.get_utf8(*descriptor_idx).to_string(),
                },
                CPInfo::Unsupported => Unsupported,
            })
            .collect::<Vec<_>>();
        RunTimeConstantPool(resolved)
    }

    pub fn empty() -> RunTimeConstantPool {
        RunTimeConstantPool(Vec::new())
    }

    pub fn get_info(&self, idx: u16) -> &RunTimeCPInfo {
        assert!(0 < idx && idx <= self.0.len() as u16);
        &self.0[idx as usize - 1]
    }
}

fn resolve_fieldref(cp: &ConstantPool, cls_idx: u16, nt_idx: u16) -> RunTimeCPInfo {
    let Some(ConstPoolRef {
        class_name,
        name,
        descriptor,
    }) = resolve_const_pool_ref(cp, cls_idx, nt_idx)
    else {
        return RunTimeCPInfo::Methodref {
            class_name: String::new(),
            name: String::new(),
            descriptor: String::new(),
        };
    };
    RunTimeCPInfo::Fieldref {
        class_name,
        name,
        descriptor,
    }
}

fn resolve_methodref(cp: &ConstantPool, cls_idx: u16, nt_idx: u16) -> RunTimeCPInfo {
    let Some(ConstPoolRef {
        class_name,
        name,
        descriptor,
    }) = resolve_const_pool_ref(cp, cls_idx, nt_idx)
    else {
        return RunTimeCPInfo::Methodref {
            class_name: String::new(),
            name: String::new(),
            descriptor: String::new(),
        };
    };
    RunTimeCPInfo::Methodref {
        class_name,
        name,
        descriptor,
    }
}

struct ConstPoolRef {
    class_name: String,
    name: String,
    descriptor: String,
}
fn resolve_const_pool_ref(cp: &ConstantPool, cls_idx: u16, nt_idx: u16) -> Option<ConstPoolRef> {
    let &CPInfo::Class { name_idx } = cp.get_info(cls_idx) else {
        eprintln!("failed to resolve methodref");
        return None;
    };
    let class_name = cp.get_utf8(name_idx).to_string();

    let &CPInfo::NameAndType {
        name_idx,
        descriptor_idx,
    } = cp.get_info(nt_idx)
    else {
        eprintln!("failed to resolve methodref");
        return None;
    };
    let name = cp.get_utf8(name_idx).to_string();
    let descriptor = cp.get_utf8(descriptor_idx).to_string();

    Some(ConstPoolRef {
        class_name,
        name,
        descriptor,
    })
}
