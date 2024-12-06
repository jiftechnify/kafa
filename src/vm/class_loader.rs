use std::{
    ffi::OsStr,
    fs::File,
    io::ErrorKind,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use zip::{result::ZipError, ZipArchive};

use crate::class_file::ClassFile;

use super::{class::Class, error::VMResult};

pub struct ClassLoader {
    classpath: Vec<PathBuf>,
}

impl ClassLoader {
    pub fn new<P>(classpath: &P) -> ClassLoader
    where
        P: AsRef<OsStr>,
    {
        ClassLoader {
            classpath: split_classpath(classpath),
        }
    }
}

impl ClassLoader {
    pub fn load(&self, name: &str) -> VMResult<Class> {
        for cp in self.classpath.iter() {
            match cp.extension() {
                Some(ext) if (ext == "jar" || ext == "zip") => match self.load_from_jar(cp, name) {
                    Ok(Some(cls)) => {
                        if cls.name == name {
                            return Ok(cls);
                        } else {
                            return Err("specified binary name and actual class name (this_class) doesn't match")?;
                        }
                    }
                    Ok(None) => continue, // class not found -> try next path
                    Err(e) => return Err(e)?,
                },
                None => match self.load_from_class(cp, name) {
                    Ok(Some(cls)) => {
                        if cls.name == name {
                            return Ok(cls);
                        } else {
                            return Err("specified binary name and actual class name (this_class) doesn't match")?;
                        }
                    }
                    Ok(None) => continue, // class not found -> try next path
                    Err(e) => return Err(e)?,
                },
                _ => continue, // skip paths other than jar/zip file or directory-ish path
            }
        }
        return Err("class '{name}' not found in classpath")?;
    }

    fn load_from_class(&self, cp: &Path, cls_name: &str) -> VMResult<Option<Class>> {
        let cls_file_path = cp.join(format!("{}.class", cls_name));

        match File::open(cls_file_path) {
            Ok(f) => {
                let cls_file = ClassFile::parse(f)?;
                Ok(Some(Class::from_class_file(cls_file)?))
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    return Err(e)?;
                }
            }
        }
    }

    fn load_from_jar(&self, jar_path: &Path, cls_name: &str) -> VMResult<Option<Class>> {
        let jar = File::open(jar_path)?;
        let mut archive = ZipArchive::new(jar)?;

        let zf = archive.by_name(&format!("{}.class", cls_name));
        match zf {
            Ok(f) => {
                let cls_file = ClassFile::parse(f)?;
                Ok(Some(Class::from_class_file(cls_file)?))
            }
            Err(ZipError::FileNotFound) => Ok(None),
            Err(e) => return Err(e)?,
        }
    }
}

// the logic is borrowed from std::env::split_paths
fn split_classpath<P>(cp: &P) -> Vec<PathBuf>
where
    P: AsRef<OsStr>,
{
    fn bytes_to_path(b: &[u8]) -> PathBuf {
        PathBuf::from(<OsStr as OsStrExt>::from_bytes(b))
    }
    fn is_separator(b: &u8) -> bool {
        *b == b';'
    }

    cp.as_ref()
        .as_bytes()
        .split(is_separator as fn(&u8) -> bool)
        .map(bytes_to_path as fn(&[u8]) -> PathBuf)
        .collect::<Vec<_>>()
}
