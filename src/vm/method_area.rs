use std::{collections::HashMap, rc::Rc};

use super::{
    class::{Class, FieldValue, Method, MethodSignature},
    class_loader::ClassLoader,
};

pub struct MethodArea {
    classes: HashMap<String, Rc<Class>>,
    loader: ClassLoader,
}

impl MethodArea {
    pub fn new(loader: ClassLoader) -> Self {
        MethodArea {
            classes: HashMap::new(),
            loader,
        }
    }
}

impl MethodArea {
    pub fn lookup_class(
        &mut self,
        class_name: &str,
    ) -> Result<Rc<Class>, Box<dyn std::error::Error>> {
        match self.classes.get(class_name) {
            Some(cls) => Ok(cls.clone()),
            None => {
                // load a .class file under the class path
                let cls = self
                    .loader
                    .load(class_name)
                    .map_err(|err| format!("failed to load '{class_name}': {err}"))?;
                let cls = Rc::new(cls);
                self.classes.insert(class_name.to_string(), cls.clone());
                Ok(cls)
            }
        }
    }

    pub fn lookup_static_field(
        &mut self,
        class_name: &str,
        name: &str,
    ) -> Result<Rc<FieldValue>, Box<dyn std::error::Error>> {
        self.lookup_class(class_name).and_then(|cls| {
            cls.lookup_static_field(name)
                .ok_or_else(|| "static field '{class_name}.{name}' not found".into())
        })
    }

    pub fn lookup_static_method(
        &mut self,
        class_name: &str,
        sig: &MethodSignature,
    ) -> Result<(Rc<Class>, Rc<Method>), Box<dyn std::error::Error>> {
        self.lookup_class(class_name).and_then(|cls| {
            cls.lookup_static_method(sig)
                .map(|meth| (cls, meth))
                .ok_or_else(|| "static method '{class_name}.{sig}' not found".into())
        })
    }
}
