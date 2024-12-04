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
    Array(Array), // TODO: consider specialized implementation for array of boolean
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

    pub fn get_class(&self) -> Rc<Class> {
        self.class.clone()
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldValue> {
        self.fields.get(name)
    }
}

pub struct Array {
    item_desc: FieldDescriptor,
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
            item_desc: FieldDescriptor::new(item_desc.to_string()),
            length,
            data,
        };
        RefValue::Array(arr)
    }

    pub fn get_length(&self) -> usize {
        self.length
    }
}
