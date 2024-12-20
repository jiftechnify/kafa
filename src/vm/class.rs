use std::{cell::Cell, collections::HashMap, hash::Hash, rc::Rc};

use crate::class_file::{
    CPInfo, ClassAccessFlags, ConstantPool, FieldInfo, MethodAccessFlags, MethodComponents,
};

use super::{
    error::VMResult, heap::Heap, method_area::MethodArea, thread::Thread, value::MutValue,
};

pub struct Class {
    pub name: String,
    const_pool: RunTimeConstantPool,

    pub access_flags: ClassAccessFlags,

    pub super_class: Option<String>,
    pub interfaces: Vec<String>,

    static_fields: HashMap<String, Rc<MutValue>>,
    static_methods: HashMap<MethodSignature, Rc<Method>>,

    inst_fields_info: Vec<FieldInfo>,
    inst_methods: HashMap<MethodSignature, Rc<Method>>,

    init_state: Cell<ClassInitState>,
}

impl Class {
    pub fn from_class_file(cls_file: crate::class_file::ClassFile) -> VMResult<Class> {
        let rtcp = RunTimeConstantPool::from_class_file_cp(cls_file.constant_pool)?;

        let mut static_fields = HashMap::new();
        let mut inst_fields_info = Vec::new();
        for f in cls_file.fields.into_iter() {
            if f.access_flags.is_static() {
                let fv = match f.get_const_val() {
                    // TODO: strictly speaking, this should be done within "initialization" process described in JVM spec 5.5.
                    Some(cp_info) => MutValue::from_cp_info(cp_info),
                    None => MutValue::default_of_type(&f.descriptor),
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
            let MethodComponents {
                access_flags,
                name,
                descriptor,
                code_attr,
            } = m.into_components();

            let sig = MethodSignature {
                name,
                descriptor: MethodDescriptor(descriptor),
            };
            let code_spec = {
                if access_flags.contains(MethodAccessFlags::ABSTRACT) {
                    MethodCodeSpec::Abstract
                } else if access_flags.contains(MethodAccessFlags::NATIVE) {
                    MethodCodeSpec::Native
                } else {
                    match code_attr {
                        Some(ca) => MethodCodeSpec::Java {
                            max_stack: ca.max_stack,
                            max_locals: ca.max_locals,
                            code: ca.code,
                        },
                        None => {
                            return Err("non-abstract & non-native methods must have Code attr")?
                        }
                    }
                }
            };
            let method = Method {
                access_flags,
                signature: sig.clone(),
                code_spec,
            };
            let method = Rc::new(method);

            if method.access_flags.is_static() {
                static_methods.insert(sig, method);
            } else {
                inst_methods.insert(sig, method);
            }
        }

        let cls = Class {
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
        };
        Ok(cls)
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
    ) -> VMResult<()> {
        use ClassInitState::*;

        let BeforeInit = self.init_state.get() else {
            return Ok(());
        };
        self.init_state.set(InProgress);

        // initialize superclass & superinterfaces that declare non-abstract & non-static methods, recursively (step 7)
        if !self.access_flags.is_interface() {
            for sc in self.superclasses_to_be_initialized(meth_area)? {
                sc.initialize(thread, meth_area, heap)?;
            }
        }

        // execute <clinit> of the class/interface (step 9)
        thread
            .exec_class_initialization(meth_area, heap, self.clone())
            .inspect(|_| {
                self.init_state.set(Succeeded);
            })
            .inspect_err(|err| {
                eprintln!("failed to initialize class '{}': {err}", &self.name);
                self.init_state.set(Failed);
            })
    }

    fn superclasses_to_be_initialized(
        &self,
        meth_area: &mut MethodArea,
    ) -> VMResult<Vec<Rc<Class>>> {
        let mut res = Vec::new();
        if let Some(ref sc_name) = self.super_class {
            let sc = meth_area.resolve_class(sc_name)?;
            res.push(sc);
        }

        // pick superinterfaces that declare non-abstract & non-static methods
        for iface_name in self.interfaces.iter() {
            let iface = meth_area.resolve_class(iface_name)?;
            if iface
                .inst_methods
                .values()
                .any(|m| !m.access_flags.contains(MethodAccessFlags::ABSTRACT))
            {
                res.push(iface);
            }
        }
        Ok(res)
    }
}

impl Class {
    pub fn lookup_static_field(&self, name: &str) -> Option<Rc<MutValue>> {
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

// Class is uniquely identified by its name (really?)
impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Class {}

impl Hash for Class {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Clone, Default)]
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

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct MethodDescriptor(String);

impl MethodDescriptor {
    fn new(raw: String) -> Self {
        Self(raw)
    }

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
    pub fn new(name: &str, desc: MethodDescriptor) -> MethodSignature {
        MethodSignature {
            name: name.to_string(),
            descriptor: desc,
        }
    }

    pub fn new_with_raw_descriptor(name: &str, desc: &str) -> MethodSignature {
        MethodSignature {
            name: name.to_string(),
            descriptor: MethodDescriptor::new(desc.to_string()),
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
pub enum MethodCodeSpec {
    Java {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
    },
    Native,
    Abstract,
}

#[derive(Clone)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub signature: MethodSignature,
    pub code_spec: MethodCodeSpec,
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
        descriptor: FieldDescriptor,
    },
    Methodref {
        class_name: String,
        name: String,
        descriptor: MethodDescriptor,
    },
    InterfaceMethodref {
        iface_name: String,
        name: String,
        descriptor: MethodDescriptor,
    },
    NameAndType {
        name: String,
        descriptor: String, // format: (<param type>*)<return type>
    },
    Unsupported,
}

impl RunTimeConstantPool {
    pub fn from_class_file_cp(
        cp: crate::class_file::ConstantPool,
    ) -> VMResult<RunTimeConstantPool> {
        use RunTimeCPInfo::*;

        let resolved = cp
            .infos()
            .map(|cp_info| match cp_info {
                CPInfo::Utf8(s) => Ok(Utf8(s.clone())),
                CPInfo::Integer(i) => Ok(Integer(*i)),
                CPInfo::Float(f) => Ok(Float(*f)),
                CPInfo::Long(l) => Ok(Long(*l)),
                CPInfo::Double(d) => Ok(Double(*d)),
                CPInfo::Class { name_idx } => Ok(Class {
                    name: cp.get_utf8(*name_idx).to_string(),
                }),
                CPInfo::String { string_idx } => Ok(String(cp.get_utf8(*string_idx).to_string())),
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
                } => Ok(NameAndType {
                    name: cp.get_utf8(*name_idx).to_string(),
                    descriptor: cp.get_utf8(*descriptor_idx).to_string(),
                }),
                CPInfo::Unsupported => Ok(Unsupported),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RunTimeConstantPool(resolved))
    }

    pub fn empty() -> RunTimeConstantPool {
        RunTimeConstantPool(Vec::new())
    }

    pub fn get_info(&self, idx: u16) -> &RunTimeCPInfo {
        assert!(0 < idx && idx <= self.0.len() as u16);
        &self.0[idx as usize - 1]
    }
}

fn resolve_fieldref(cp: &ConstantPool, cls_idx: u16, nt_idx: u16) -> VMResult<RunTimeCPInfo> {
    let ConstPoolRef {
        class_name,
        name,
        descriptor,
    } = resolve_const_pool_ref(cp, cls_idx, nt_idx)?;

    let fld = RunTimeCPInfo::Fieldref {
        class_name,
        name,
        descriptor: FieldDescriptor(descriptor),
    };
    Ok(fld)
}

fn resolve_methodref(cp: &ConstantPool, cls_idx: u16, nt_idx: u16) -> VMResult<RunTimeCPInfo> {
    let ConstPoolRef {
        class_name,
        name,
        descriptor,
    } = resolve_const_pool_ref(cp, cls_idx, nt_idx)?;

    let meth = RunTimeCPInfo::Methodref {
        class_name,
        name,
        descriptor: MethodDescriptor(descriptor),
    };
    Ok(meth)
}

fn resolve_interface_methodref(
    cp: &ConstantPool,
    cls_idx: u16,
    nt_idx: u16,
) -> VMResult<RunTimeCPInfo> {
    let ConstPoolRef {
        class_name: iface_name,
        name,
        descriptor,
    } = resolve_const_pool_ref(cp, cls_idx, nt_idx)?;
    let meth = RunTimeCPInfo::InterfaceMethodref {
        iface_name,
        name,
        descriptor: MethodDescriptor(descriptor),
    };
    Ok(meth)
}

struct ConstPoolRef {
    class_name: String,
    name: String,
    descriptor: String,
}
fn resolve_const_pool_ref(cp: &ConstantPool, cls_idx: u16, nt_idx: u16) -> VMResult<ConstPoolRef> {
    let &CPInfo::Class { name_idx } = cp.get_info(cls_idx) else {
        return Err("failed to resolve CPInfo::Class".into());
    };
    let class_name = cp.get_utf8(name_idx).to_string();

    let &CPInfo::NameAndType {
        name_idx,
        descriptor_idx,
    } = cp.get_info(nt_idx)
    else {
        return Err("failed to resolve CPInfo::NameAndType".into());
    };
    let name = cp.get_utf8(name_idx).to_string();
    let descriptor = cp.get_utf8(descriptor_idx).to_string();

    Ok(ConstPoolRef {
        class_name,
        name,
        descriptor,
    })
}
