use std::{collections::HashMap, rc::Rc};

use super::{
    class::{Class, FieldDescriptor},
    method_area::MethodArea,
    value::{MutValue, Value},
};

pub struct Heap(Vec<RefValue>);

impl Heap {
    pub fn new() -> Self {
        // Value::Reference(0) -> null
        Heap(vec![RefValue::Null])
    }
}

impl Heap {
    pub fn alloc_object(&mut self, class: Rc<Class>, meth_area: &mut MethodArea) -> Value {
        let obj = Object::new(class, meth_area);
        self.alloc_ref_val(RefValue::Object(obj))
    }

    pub fn alloc_array(&mut self, len: u32, item_desc: &str) -> Value {
        let arr = new_array_of_type(len, item_desc);
        self.alloc_ref_val(RefValue::Array(arr))
    }

    fn alloc_ref_val(&mut self, rv: RefValue) -> Value {
        self.0.push(rv);

        // reference to allocated value = index of the value in underlying vec
        Value::Reference(self.0.len() - 1)
    }
}

impl Heap {
    pub fn get(&mut self, r: usize) -> Option<&mut RefValue> {
        self.0.get_mut(r)
    }
}

pub enum RefValue {
    Object(Object),
    Array(Box<dyn JavaArray>),
    Null,
}

impl RefValue {
    const SUPERTYPES_OF_ARRAY: [&str; 3] = [
        "java/lang/Object",
        "java/lang/Cloneable",
        "java/io/Serializable",
    ];
    pub fn is_instance_of(&self, target_cls_name: &str, meth_area: &MethodArea) -> bool {
        match self {
            RefValue::Object(obj) => meth_area.is_subclass_of(&obj.class.name, target_cls_name),
            RefValue::Array(arr) => {
                let arr_desc = arr.descriptor();
                let mut sc = arr_desc.as_str();
                let mut tc = target_cls_name;

                // peel array prefix ('[') one by one until either of them hit non-array component
                while sc.starts_with("[") && tc.starts_with("[") {
                    sc = &sc[1..];
                    tc = &tc[1..];
                }

                // SC is array type -> TC is a supertype of any array type?
                if sc.starts_with("[") && tc.starts_with("L") {
                    let targ_cls_name = &tc[1..]; // remove prefix 'L'
                    return RefValue::SUPERTYPES_OF_ARRAY
                        .iter()
                        .any(|st| targ_cls_name == *st);
                }
                // TC and SC are reference types -> SC can be cast to TC?
                if sc.starts_with("L") && tc.starts_with("L") {
                    return meth_area.is_subclass_of(&sc[1..], &tc[1..]);
                }
                // TC and SC are the same primitive type?
                sc.len() == 1 && tc.len() == 1 && sc == tc
            }
            RefValue::Null => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct InstanceFieldIdent {
    class_name: String,
    name: String,
}

impl InstanceFieldIdent {
    pub fn new(cls_name: &str, name: &str) -> InstanceFieldIdent {
        Self {
            class_name: cls_name.to_string(),
            name: name.to_string(),
        }
    }
}

pub struct Object {
    class: Rc<Class>,
    fields: HashMap<InstanceFieldIdent, MutValue>,
}

impl Object {
    fn new(base_cls: Rc<Class>, meth_area: &mut MethodArea) -> Self {
        let mut fields = HashMap::new();

        let mut classes = meth_area
            .collect_all_superclasses(&base_cls.name)
            .inspect_err(|err| eprintln!("{}", err))
            .unwrap();
        classes.push(base_cls.clone());

        classes.into_iter().for_each(|cls| {
            for f in cls.instance_fields() {
                let id = InstanceFieldIdent::new(&cls.name, &f.name);
                let fv = MutValue::default_of_type(&f.descriptor);
                fields.insert(id, fv);
            }
        });

        Object {
            class: base_cls,
            fields,
        }
    }

    pub fn get_class(&self) -> Rc<Class> {
        self.class.clone()
    }

    pub fn get_field(&self, cls_name: &str, fld_name: &str) -> Option<&MutValue> {
        let id = InstanceFieldIdent::new(cls_name, fld_name);
        self.fields.get(&id)
    }
}

pub trait JavaArray {
    fn get(&self, idx: u32) -> Option<Value>;
    fn put(&mut self, idx: u32, v: Value);
    fn len(&self) -> u32;
    fn descriptor(&self) -> FieldDescriptor;
}

fn new_array_of_type(len: u32, item_desc: &str) -> Box<dyn JavaArray> {
    let Some(fst_char) = item_desc.chars().next() else {
        unreachable!()
    };
    match fst_char {
        'Z' => JavaBooleanArray::new(len),
        'C' => JavaCharArray::new(len),
        'F' => JavaFloatArray::new(len),
        'D' => JavaDoubleArray::new(len),
        'B' => JavaByteArray::new(len),
        'S' => JavaShortArray::new(len),
        'I' => JavaIntArray::new(len),
        'J' => JavaLongArray::new(len),
        'L' | '[' => JavaReferenceArray::new(len, item_desc),
        _ => unreachable!(),
    }
}

struct JavaReferenceArray {
    data: Box<[MutValue]>,
    l: u32,
    desc: FieldDescriptor,
}

impl JavaReferenceArray {
    #[allow(clippy::new_ret_no_self)]
    fn new(len: u32, item_desc: &str) -> Box<dyn JavaArray> {
        let null = MutValue::from_val(Value::Reference(0));
        let data = vec![null; len as usize];
        let data = data.into(); // Vec<MutValue> -> Box<[MutValue]]>

        let arr = JavaReferenceArray {
            // prepend '[' to item's descriptor
            desc: FieldDescriptor::new(format!("[{}", item_desc)),
            l: len,
            data,
        };
        Box::new(arr)
    }
}

impl JavaArray for JavaReferenceArray {
    fn get(&self, idx: u32) -> Option<Value> {
        self.data.get(idx as usize).map(|fv| fv.get())
    }

    fn put(&mut self, idx: u32, v: Value) {
        self.data.get(idx as usize).inspect(|fv| fv.put(v));
    }

    fn len(&self) -> u32 {
        self.l
    }

    fn descriptor(&self) -> FieldDescriptor {
        self.desc.clone()
    }
}

struct JavaPrimitiveArray<T> {
    data: Box<[T]>,
    l: u32,
}

macro_rules! java_prim_array {
    ($arr_type_name:ident, $item_type: ty, $zero: literal, $val_type: path, $val_inner_type: ty, $desc: literal) => {
        type $arr_type_name = JavaPrimitiveArray<$item_type>;

        impl $arr_type_name {
            #[allow(clippy::new_ret_no_self)]
            fn new(len: u32) -> Box<dyn JavaArray> {
                let data = vec![$zero; len as usize].into();
                Box::new(Self { data, l: len })
            }
        }

        impl JavaArray for $arr_type_name {
            fn get(&self, idx: u32) -> Option<Value> {
                self.data
                    .get(idx as usize)
                    .map(|n| $val_type(*n as $val_inner_type))
            }

            fn put(&mut self, idx: u32, v: Value) {
                let $val_type(n) = v else {
                    unreachable!();
                };
                self.data[idx as usize] = n as $item_type
            }

            fn len(&self) -> u32 {
                self.l
            }

            fn descriptor(&self) -> FieldDescriptor {
                FieldDescriptor::new($desc.into())
            }
        }
    };
}

java_prim_array!(JavaByteArray, i8, 0, Value::Int, i32, "[B");
java_prim_array!(JavaShortArray, i16, 0, Value::Int, i32, "[S");
java_prim_array!(JavaIntArray, i32, 0, Value::Int, i32, "[I");
java_prim_array!(JavaLongArray, i64, 0, Value::Long, i64, "[J");
java_prim_array!(JavaCharArray, u16, 0, Value::Int, i32, "[C");
java_prim_array!(JavaFloatArray, f32, 0.0, Value::Float, f32, "[F");
java_prim_array!(JavaDoubleArray, f64, 0.0, Value::Double, f64, "[D");

// use compacted bitarray for boolean[]
struct JavaBooleanArray {
    data: Box<[u8]>,
    l: u32,
}

impl JavaBooleanArray {
    #[allow(clippy::new_ret_no_self)]
    fn new(len: u32) -> Box<dyn JavaArray> {
        let n_bytes = if len == 0 { 0 } else { (len - 1) / 8 + 1 };
        let data = vec![0u8; n_bytes as usize];
        let data = data.into();
        Box::new(Self { data, l: len })
    }
}

impl JavaArray for JavaBooleanArray {
    fn get(&self, idx: u32) -> Option<Value> {
        if idx >= self.l {
            return None;
        }
        let (i, s) = (idx / 8, (7 - idx % 8));
        let v = (self.data[i as usize] >> s) & 1;
        Some(Value::Int(v as i32))
    }

    fn put(&mut self, idx: u32, v: Value) {
        let Value::Int(n) = v else { unreachable!() };
        let tf = (n & 1) == 1; // lsb == 0 -> false, 1 -> true

        let (i, s) = (idx / 8, (7 - idx % 8));
        let i = i as usize;
        assert!(i < self.data.len());

        let mask = 1 << s;

        match tf {
            true => self.data[i] |= mask,
            false => self.data[i] &= !mask,
        }
    }

    fn len(&self) -> u32 {
        self.l
    }

    fn descriptor(&self) -> FieldDescriptor {
        FieldDescriptor::new("[Z".into())
    }
}
