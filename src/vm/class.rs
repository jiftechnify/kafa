use std::{cell::Cell, collections::HashMap, rc::Rc};

use crate::class_file::{
    CPInfo, ClassAccessFlags, ConstantPool, FieldInfo, MethodAccessFlags, MethodComponents,
};

use super::{heap::Heap, method_area::MethodArea, thread::Thread, Value};

pub struct Class {
    pub name: String,
    const_pool: RunTimeConstantPool,

    pub access_flags: ClassAccessFlags,

    pub super_class: Option<String>,
    pub interfaces: Vec<String>,

    static_fields: HashMap<String, Rc<FieldValue>>,
    static_methods: HashMap<MethodSignature, Rc<Method>>,

    inst_fields_info: Vec<FieldInfo>,
    inst_methods: HashMap<MethodSignature, Rc<Method>>,

    init_state: Cell<ClassInitState>,
}

impl Class {
    pub fn from_class_file(cls_file: crate::class_file::ClassFile) -> Class {
        let rtcp = RunTimeConstantPool::from_class_file_cp(cls_file.constant_pool);

        let mut static_fields = HashMap::new();
        let mut inst_fields_info = Vec::new();
        for f in cls_file.fields.into_iter() {
            if f.access_flags.is_static() {
                let fv = match f.get_const_val() {
                    // TODO: strictly speaking, this should be done within "initialization" process described in JVM spec 5.5.
                    Some(cp_info) => FieldValue::from_cp_info(cp_info),
                    None => FieldValue::default_val_of_type(&f.descriptor),
                };
                let fv = Rc::new(fv);
                static_fields.insert(f.name, fv);
            } else {
                inst_fields_info.push(f)
            }
        }

        let mut static_methods = HashMap::new();
        let mut inst_methods = HashMap::new();
        for m in cls_file.methods.into_iter() {
            let is_static = m.is_static();

            let (name, desc, max_stack, max_locals, code) = m.into_components();
            let sig = MethodSignature {
                name,
                descriptor: MethodDescriptor(desc),
            };
            let method = Method {
                access_flags,
                signature: sig.clone(),
                max_stack,
                max_locals,
                code,
            };
            let method = Rc::new(method);

            if method.access_flags.is_static() {
                static_methods.insert(sig, method);
            } else {
                inst_methods.insert(sig, method);
            }
        }

        Class {
            name: cls_file.this_class,
            const_pool: rtcp,
            access_flags: cls_file.access_flags,
            super_class: cls_file.super_class,
            interfaces: cls_file.interfaces,
            static_fields,
            static_methods,
            inst_fields_info,
            inst_methods,
            init_state: Cell::new(ClassInitState::BeforeInit),
        }
    }

    pub fn dummy() -> Class {
        Class {
            name: "dummy".to_string(),
            const_pool: RunTimeConstantPool::empty(),
            access_flags: ClassAccessFlags::empty(),
            super_class: None,
            interfaces: Vec::new(),
            static_fields: HashMap::new(),
            static_methods: HashMap::new(),
            inst_fields_info: Vec::new(),
            inst_methods: HashMap::new(),
            init_state: Cell::new(ClassInitState::BeforeInit),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ClassInitState {
    BeforeInit,
    InProgress,
    Succeeded,
    Failed,
}

impl Class {
    // executes class initialization steps described in JVM spec 5.5.
    pub(in crate::vm) fn initialize(
        self: Rc<Self>,
        thread: &mut Thread,
        meth_area: &mut MethodArea,
        heap: &mut Heap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use ClassInitState::*;

        let BeforeInit = self.init_state.get() else {
            return Ok(());
        };
        self.init_state.set(InProgress);
        thread
            .exec_class_initialization(meth_area, heap, self.clone())
            .inspect(|_| {
                self.init_state.set(Succeeded);
            })
            .inspect_err(|_| {
                self.init_state.set(Failed);
            })
    }
}

impl Class {
    pub fn lookup_static_field(&self, name: &str) -> Option<Rc<FieldValue>> {
        self.static_fields.get(name).cloned()
    }

    pub fn lookup_static_method(&self, signature: &MethodSignature) -> Option<Rc<Method>> {
        self.static_methods.get(signature).cloned()
    }

    pub fn lookup_instance_method(&self, signature: &MethodSignature) -> Option<Rc<Method>> {
        self.inst_methods.get(signature).cloned()
    }

    pub fn instance_fields(&self) -> impl Iterator<Item = &FieldInfo> {
        self.inst_fields_info.iter()
    }

    pub fn get_cp_info(&self, idx: u16) -> &RunTimeCPInfo {
        self.const_pool.get_info(idx)
    }
}

#[derive(Clone)]
pub struct FieldDescriptor(String);

impl FieldDescriptor {
    pub fn new(raw: String) -> FieldDescriptor {
        Self(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FieldDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct FieldValue(Cell<Value>);

impl FieldValue {
    fn from_val(val: Value) -> Self {
        FieldValue(Cell::new(val))
    }

    fn from_cp_info(cp_info: &CPInfo) -> Self {
        use CPInfo::*;
        let val = match cp_info {
            Integer(i) => Value::Int(*i),
            Float(f) => Value::Float(*f),
            Long(l) => Value::Long(*l),
            Double(d) => Value::Double(*d),
            String { .. } => {
                eprintln!("string constant is not supported");
                Value::Reference(0) // null
            }
            _ => {
                eprintln!("not a constant value");
                Value::Reference(0) // null
            }
        };
        FieldValue::from_val(val)
    }

    pub fn default_val_of_type(desc: &str) -> Self {
        assert!(!desc.is_empty());
        let Some(fst_char) = desc.chars().nth(0) else {
            unreachable!()
        };
        let default_val = match fst_char {
            'B' => Value::Byte(0),
            'C' => Value::Char(0),
            'D' => Value::Double(0.0),
            'F' => Value::Float(0.0),
            'I' => Value::Int(0),
            'J' => Value::Long(0),
            'L' => Value::Reference(0), // null
            'S' => Value::Short(0),
            // 'Z' -> boolean
            // the Java programming language that operate on boolean values
            // are compiled to use values of the Java Virtual Machine int data type.
            'Z' => Value::Int(0),
            '[' => Value::Reference(0), // null reference to arrays
            _ => unreachable!(),
        };
        FieldValue::from_val(default_val)
    }
}

impl FieldValue {
    pub fn get(&self) -> Value {
        self.0.get()
    }

    pub fn put(&self, new_val: Value) {
        self.0.set(new_val);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MethodDescriptor(String);

impl MethodDescriptor {
    fn num_args(&self) -> usize {
        assert!(!self.0.is_empty());

        let mut n = 0;
        let mut in_ref = false; // is reading ref type?
        for c in self.0[1..].chars() {
            match (in_ref, c) {
                (_, ')') => {
                    break;
                }
                (_, '[') => {
                    // skip prefix for array type
                    continue;
                }
                (false, 'L') => {
                    // enter ref type
                    in_ref = true;
                }
                (false, _) => {
                    // primitive type
                    n += 1;
                }
                (true, ';') => {
                    // leave ref type
                    n += 1;
                    in_ref = false;
                }
                (true, _) => {
                    // skip letters in ref type
                    continue;
                }
            }
        }
        n
    }
}

#[cfg(test)]
mod test_method_descriptor {
    use super::*;

    #[test]
    fn test_num_args() {
        let tests = vec![
            ("()V", 0),
            ("(I)V", 1),
            ("(ISB)V", 3),
            ("(Ljava/lang/String;)V", 1),
            ("(Ljava/lang/String;ILjava/lang/String;)V", 3),
            ("([I)", 1),
            ("([[I[I)", 2),
            ("([Ljava/lang/String;)V", 1),
            ("([[Ljava/lang/String;I)V", 2),
            ("(I[[ILjava/lang/String;[II[Ljava/lang/String;I)V", 7),
        ];

        for (input, exp) in tests {
            let desc = MethodDescriptor(input.to_string());
            assert_eq!(desc.num_args(), exp);
        }
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
    pub access_flags: MethodAccessFlags,
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
    InterfaceMethodref {
        iface_name: String,
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
                CPInfo::InterfaceMethodref {
                    class_idx,
                    name_and_type_idx,
                } => resolve_interface_methodref(&cp, *class_idx, *name_and_type_idx),
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

fn resolve_interface_methodref(cp: &ConstantPool, cls_idx: u16, nt_idx: u16) -> RunTimeCPInfo {
    let Some(ConstPoolRef {
        class_name: iface_name,
        name,
        descriptor,
    }) = resolve_const_pool_ref(cp, cls_idx, nt_idx)
    else {
        return RunTimeCPInfo::InterfaceMethodref {
            iface_name: String::new(),
            name: String::new(),
            descriptor: String::new(),
        };
    };
    RunTimeCPInfo::InterfaceMethodref {
        iface_name,
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
