use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::class_file::MethodAccessFlags;

use super::{
    class::{Class, FieldValue, Method, MethodSignature},
    class_loader::ClassLoader,
    error::VMResult,
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
    pub fn resolve_class(&mut self, class_name: &str) -> VMResult<Rc<Class>> {
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

    pub fn is_subclass_of(&self, cls_name: &str, target_cls_name: &str) -> bool {
        let cls = self
            .classes
            .get(cls_name)
            .expect("class must have been resolved");

        if cls.name == target_cls_name {
            return true;
        }

        // traverse superclasses
        if cls
            .super_class
            .iter()
            .any(|sc_name| self.is_subclass_of(sc_name, target_cls_name))
        {
            return true;
        }
        // traverse superinterfaces
        cls.interfaces
            .iter()
            .any(|if_name| self.is_subclass_of(if_name, target_cls_name))
    }

    pub fn collect_all_superclasses(&self, class_name: &str) -> VMResult<Vec<Rc<Class>>> {
        let Some(c) = self.classes.get(class_name) else {
            return Err("class '{class_name}' have not been resolved")?;
        };

        // ignoring warning, since sc_set is just temporal for deduplication
        #[allow(clippy::mutable_key_type)]
        let mut sc_set = HashSet::new();
        if let Some(sc_name) = &c.super_class {
            let scs = self.collect_all_superclasses(sc_name)?;
            for sc in scs {
                sc_set.insert(sc);
            }
        }
        for iface_name in &c.interfaces {
            let scs = self.collect_all_superclasses(iface_name)?;
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
    ) -> VMResult<Rc<FieldValue>> {
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
    ) -> VMResult<Option<Rc<FieldValue>>> {
        let cls = self.resolve_class(class_name)?;
        Ok(cls.lookup_static_field(name))
    }

    pub fn resolve_static_method(
        &mut self,
        class_name: &str,
        sig: &MethodSignature,
    ) -> VMResult<(Rc<Class>, Rc<Method>)> {
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

    pub fn resolve_instance_method(
        &mut self,
        class_name: &str,
        sig: &MethodSignature,
    ) -> VMResult<Rc<Method>> {
        // the symbolic reference to C given by the method reference is first resolved.
        let cls = self.resolve_class(class_name)?;

        if cls.access_flags.is_interface() {
            self.resolve_instance_method_interface(&cls.name, sig)
        } else {
            self.resolve_instance_method_class(&cls.name, sig)
        }
        .map_err(|err| {
            format!("failed to resolve instance method '{class_name}.{sig}': {err}").into()
        })
    }

    fn resolve_instance_method_class(
        &mut self,
        cls_name: &str,
        sig: &MethodSignature,
    ) -> VMResult<Rc<Method>> {
        let cls = self.classes.get(cls_name).expect("").clone();
        assert!(!cls.access_flags.is_interface());

        if let Some(m) = cls.lookup_instance_method(sig) {
            return Ok(m);
        }

        if let Some(sc_name) = &cls.super_class {
            if let Ok(m) = self.resolve_instance_method_class(sc_name, sig) {
                return Ok(m);
            }
        }

        self.find_maximally_specific_superinterface_method(&cls, sig)
    }

    fn resolve_instance_method_interface(
        &mut self,
        iface_name: &str,
        sig: &MethodSignature,
    ) -> VMResult<Rc<Method>> {
        let iface = self.classes.get(iface_name).expect("").clone();
        assert!(iface.access_flags.is_interface());
        if let Some(m) = iface.lookup_instance_method(sig) {
            return Ok(m);
        }

        if let Ok(m) = self.resolve_instance_method_class("java/lang/Object", sig) {
            if m.access_flags.is_public_non_static() {
                return Ok(m);
            }
        }

        self.find_maximally_specific_superinterface_method(&iface, sig)
    }

    // select method to be called, based on the algorithm specified in JVM spec 5.4.6.
    pub fn select_instance_method(
        &mut self,
        runtime_class: &Class,
        resolved_meth: Rc<Method>,
    ) -> VMResult<Rc<Method>> {
        // 1. If mR is marked ACC_PRIVATE, then it is the selected method.
        if resolved_meth
            .access_flags
            .contains(MethodAccessFlags::PRIVATE)
        {
            return Ok(resolved_meth);
        }

        let sig = &resolved_meth.signature;

        // 2. Otherwise, the selected method is determined by the following lookup procedure:
        let mut cls = runtime_class;
        loop {
            // If C contains a declaration of an instance method m that can override mR, then m is the selected method.
            // TODO: "transitive overriding" of methods with default access is not taken account for now.
            if let Some(meth) = runtime_class.lookup_instance_method(sig) {
                return Ok(meth);
            }
            // Otherwise, if C has a superclass, a search for a declaration of an instance method that can override mR is performed,
            // starting with the direct superclass of C and continuing with the direct superclass of that class,
            // and so forth, until a method is found or no further superclasses exist.
            // If a method is found, it is the selected method.
            match &cls.super_class {
                Some(sc_name) => {
                    cls = self
                        .classes
                        .get(sc_name)
                        .expect("all superclasses must have been resolved");
                }
                None => break,
            }
        }
        // Otherwise, the maximally-specific superinterface methods of C are determined.
        // If exactly one matches mR's name and descriptor and is not abstract, then it is the selected method.
        self.find_maximally_specific_superinterface_method(runtime_class, sig)
    }

    fn find_maximally_specific_superinterface_method(
        &self,
        base: &Class,
        sig: &MethodSignature,
    ) -> VMResult<Rc<Method>> {
        let mut seen_ifaces = HashSet::new();
        let mut ifaces = base
            .interfaces
            .iter()
            .map(|iface_name| {
                seen_ifaces.insert(iface_name);
                self.classes.get(iface_name)
            })
            .collect::<Option<Vec<_>>>()
            .expect("all superinterfaces must have been resolved");

        loop {
            let candidates = ifaces
                .iter()
                .filter_map(|&iface| {
                    iface
                        .lookup_instance_method(sig)
                        .filter(|meth| meth.access_flags.is_interface_default())
                })
                .collect::<Vec<_>>();

            if candidates.len() == 1 {
                return Ok(candidates[0].clone());
            } else if candidates.len() >= 2 {
                return Err("maximally-specific superinterface method not found".into());
            } else {
                // no candidates -> travarse next level up
                let next = ifaces
                    .iter()
                    .flat_map(|&iface| {
                        iface
                            .interfaces // names of superinterfaces
                            .iter()
                            .filter_map(|iface_name| {
                                // already seen interfaces can be ignored safely,
                                // since they shouldn't have method that matches with sig.
                                let seen = seen_ifaces.insert(iface_name);
                                if seen {
                                    None
                                } else {
                                    self.classes.get(iface_name)
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                if next.is_empty() {
                    return Err("maximally-specific superinterface method not found".into());
                }
                ifaces = next;
            }
        }
    }
}
