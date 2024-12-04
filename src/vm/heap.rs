use std::{collections::HashMap, rc::Rc};

use super::{
    class::{Class, FieldDescriptor, FieldValue},
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
    pub fn alloc_object(&mut self, class: Rc<Class>) -> Value {
        let obj = Object::new(class);
        self.alloc_ref_val(obj)
    }

    pub fn alloc_array(&mut self, length: usize, item_desc: &FieldDescriptor) -> Value {
        let arr = Array::new(length, item_desc);
        self.alloc_ref_val(arr)
    }

    fn alloc_ref_val(&mut self, rv: RefValue) -> Value {
        self.0.push(rv);

        // reference to allocated value = index of the value in underlying vec
        Value::Reference(self.0.len() - 1)
    }
}

pub enum RefValue {
    Object(Object),
    Array(Array),
    Null,
}

pub struct Object {
    class: Rc<Class>,
    fields: HashMap<String, FieldValue>,
}

impl Object {
    #[allow(clippy::new_ret_no_self)]
    fn new(class: Rc<Class>) -> RefValue {
        let mut fields = HashMap::new();
        for f in class.instance_fields() {
            let fv = FieldValue::default_val_of_type(&f.descriptor);
            fields.insert(f.name.clone(), fv);
        }

        let obj = Object { class, fields };
        RefValue::Object(obj)
    }
}

pub struct Array {
    item_desc: FieldDescriptor,
    length: usize,
    data: Box<[FieldValue]>,
}

impl Array {
    #[allow(clippy::new_ret_no_self)]
    fn new(length: usize, item_desc: &FieldDescriptor) -> RefValue {
        let item_desc = item_desc.clone();

        let mut data = Vec::with_capacity(length);
        data.resize_with(length, || {
            FieldValue::default_val_of_type(item_desc.as_str())
        });
        let data = data.into(); // Vec<FieldValue> -> Box<[FieldValue]]>

        let arr = Array {
            item_desc,
            length,
            data,
        };
        RefValue::Array(arr)
    }
}