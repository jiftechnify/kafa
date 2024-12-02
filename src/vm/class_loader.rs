use std::{ffi::OsStr, fs::File, path::PathBuf};

use crate::class_file::ClassFile;

use super::class::Class;

pub struct ClassLoader {
    class_path: PathBuf,
}

impl ClassLoader {
    pub fn new<P>(class_path: &P) -> ClassLoader
    where
        P: AsRef<OsStr>,
    {
        ClassLoader {
            class_path: PathBuf::from(class_path),
        }
    }
}

impl ClassLoader {
    pub fn load(&self, name: &str) -> Result<Class, Box<dyn std::error::Error>> {
        let cls_file_path = self.class_path.join(format!("{}.class", name));
        let f = File::open(cls_file_path)?;
        let cls_file = ClassFile::parse(f)?;
        Ok(Class::from_class_file(cls_file))
    }
}
