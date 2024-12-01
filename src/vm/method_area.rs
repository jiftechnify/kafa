use std::{collections::HashMap, rc::Rc};

use super::class::{Class, Method, MethodSignature};

pub struct MethodArea {
    classes: HashMap<String, Rc<Class>>,
}

impl MethodArea {
    pub fn new() -> Self {
        MethodArea {
            classes: HashMap::new(),
        }
    }

    pub fn with_class(cls: Rc<Class>) -> Self {
        let mut classes = HashMap::new();
        classes.insert(cls.name.clone(), cls);
        MethodArea { classes }
    }
}

impl MethodArea {
    // TODO: dynamic class loading
    pub fn lookup_class(&mut self, class_name: &str) -> Option<Rc<Class>> {
        self.classes.get(class_name).cloned()
    }

    pub fn lookup_static_method(
        &mut self,
        class_name: &str,
        signature: &MethodSignature,
    ) -> Option<Method> {
        self.lookup_class(class_name)
            .and_then(|cls| cls.lookup_static_method(signature))
    }
}
