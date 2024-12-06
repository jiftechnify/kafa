use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

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
    pub fn resolve_class(
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

                // resolve the super class / interfaces
                if let Some(super_cls_name) = &cls.super_class {
                    self.resolve_class(super_cls_name)?;
                }
                for iface_name in &cls.interfaces {
                    self.resolve_class(iface_name)?;
                }

                let cls = Rc::new(cls);
                self.classes.insert(class_name.to_string(), cls.clone());
                Ok(cls)
            }
        }
    }

    pub fn lookup_all_superclasses(
        &self,
        class_name: &str,
    ) -> Result<Vec<Rc<Class>>, Box<dyn std::error::Error>> {
        let Some(c) = self.classes.get(class_name) else {
            return Err("class '{class_name}' have not been resolved")?;
        };

        // ignoring warning, since sc_set is just temporal for deduplication
        #[allow(clippy::mutable_key_type)]
        let mut sc_set = HashSet::new();
        if let Some(sc_name) = &c.super_class {
            let scs = self.lookup_all_superclasses(sc_name)?;
            for sc in scs {
                sc_set.insert(sc);
            }
        }
        for iface_name in &c.interfaces {
            let scs = self.lookup_all_superclasses(iface_name)?;
            for sc in scs {
                sc_set.insert(sc);
            }
        }
        Ok(sc_set.into_iter().collect())
    }

    pub fn resolve_static_field(
        &mut self,
        class_name: &str,
        name: &str,
    ) -> Result<Rc<FieldValue>, Box<dyn std::error::Error>> {
        // the symbolic reference to C given by the field reference must first be resolved.
        let cls = self.resolve_class(class_name)?;

        // 1. If C declares a field with the name and descriptor specified by the field reference, field lookup succeeds.
        if let Some(f) = cls.lookup_static_field(name) {
            return Ok(f);
        }

        // 2. Otherwise, field lookup is applied recursively to the direct superinterfaces of the specified class or interface C.
        for iface_name in &cls.interfaces {
            if let Some(f) = self.maybe_resolve_static_field(iface_name, name)? {
                return Ok(f);
            }
        }

        // 3. Otherwise, if C has a superclass S, field lookup is applied recursively to S.
        if let Some(sc_name) = &cls.super_class {
            if let Some(f) = self.maybe_resolve_static_field(sc_name, name)? {
                return Ok(f);
            }
        }

        // 4. Otherwise, field lookup fails.
        Err("static field '{class_name}.{name}' not found")?
    }

    fn maybe_resolve_static_field(
        &mut self,
        class_name: &str,
        name: &str,
    ) -> Result<Option<Rc<FieldValue>>, Box<dyn std::error::Error>> {
        let cls = self.resolve_class(class_name)?;
        Ok(cls.lookup_static_field(name))
    }

    pub fn resolve_static_method(
        &mut self,
        class_name: &str,
        sig: &MethodSignature,
    ) -> Result<(Rc<Class>, Rc<Method>), Box<dyn std::error::Error>> {
        // the symbolic reference to C given by the method reference is first resolved.
        let cls = self.resolve_class(class_name)?;

        // static interface method resolution (a portion of JVM spec 5.4.3.4.)
        // static interface method is never inherited, so it's enough.
        if cls.access_flags.is_interface() {
            return cls
                .lookup_static_method(sig)
                .map(|meth| (cls.clone(), meth))
                .ok_or("static method '{class_name}.{sig}' not found".into());
        }

        // static method resolution (JVM spec 5.4.3.3.)
        // static interface method is never inherited, so no need to search superinterfaces.

        // 2. (Otherwise,) if C declares a method with the name and descriptor specified by the method reference, method lookup succeeds.
        if let Some(cm) = cls
            .lookup_static_method(sig)
            .map(|meth| (cls.clone(), meth))
        {
            return Ok(cm);
        }
        // Otherwise, if C has a superclass, step 2 of method resolution is recursively invoked on the direct superclass of C.
        if let Some(sc_name) = &cls.super_class {
            if let Ok(cm) = self.resolve_static_method(sc_name, sig) {
                return Ok(cm);
            }
        }
        Err("static method '{class_name}.{sig}' not found")?
    }

    pub fn lookup_instance_method(
        &mut self,
        _resolved_class_name: &str,
        runtime_class: &Class,
        sig: &MethodSignature,
    ) -> Result<Rc<Method>, Box<dyn std::error::Error>> {
        // TODO: implement method selection algorithm that take account of override relations (JVM spec 5.4.6.)
        runtime_class
            .lookup_instance_method(sig)
            .ok_or_else(|| "instance method '{runtime_class.name}.{sig} not found'".into())
    }
}
