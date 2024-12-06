use std::{collections::HashMap, rc::Rc};

use super::{
    class::{Class, FieldDescriptor, FieldValue},
    method_area::MethodArea,
    Value,
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
        self.alloc_ref_val(obj)
    }

    pub fn alloc_array(&mut self, length: usize, item_desc: &str) -> Value {
        let arr = Array::new(length, item_desc);
        self.alloc_ref_val(arr)
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
    Array(Array), // TODO: consider specialized implementation for array of primitive types
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
                let mut sc = arr.desc.as_str();
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
    fields: HashMap<InstanceFieldIdent, FieldValue>,
}

impl Object {
    #[allow(clippy::new_ret_no_self)]
    fn new(base_cls: Rc<Class>, meth_area: &mut MethodArea) -> RefValue {
        let mut fields = HashMap::new();

        let mut classes = meth_area
            .collect_all_superclasses(&base_cls.name)
            .inspect_err(|err| eprintln!("{}", err))
            .unwrap();
        classes.push(base_cls.clone());

        classes.into_iter().for_each(|cls| {
            for f in cls.instance_fields() {
                let id = InstanceFieldIdent::new(&cls.name, &f.name);
                let fv = FieldValue::default_val_of_type(&f.descriptor);
                fields.insert(id, fv);
            }
        });

        let obj = Object {
            class: base_cls,
            fields,
        };
        RefValue::Object(obj)
    }

    pub fn get_class(&self) -> Rc<Class> {
        self.class.clone()
    }

    pub fn get_field(&self, cls_name: &str, fld_name: &str) -> Option<&FieldValue> {
        let id = InstanceFieldIdent::new(cls_name, fld_name);
        self.fields.get(&id)
    }
}

pub struct Array {
    desc: FieldDescriptor,
    length: usize,
    data: Box<[FieldValue]>,
}

impl Array {
    #[allow(clippy::new_ret_no_self)]
    fn new(length: usize, item_desc: &str) -> RefValue {
        let mut data = Vec::with_capacity(length);
        let v = FieldValue::default_val_of_type(item_desc);
        data.resize(length, v);
        let data = data.into(); // Vec<FieldValue> -> Box<[FieldValue]]>

        let arr = Array {
            // prepend '[' to item's descriptor
            desc: FieldDescriptor::new(format!("[{}", item_desc)),
            length,
            data,
        };
        RefValue::Array(arr)
    }

    pub fn get(&self, idx: usize) -> Option<Value> {
        self.data.get(idx).map(|fv| fv.get())
    }

    pub fn put(&mut self, idx: usize, v: Value) {
        self.data.get(idx).inspect(|fv| fv.put(v));
    }

    pub fn get_length(&self) -> usize {
        self.length
    }
}
