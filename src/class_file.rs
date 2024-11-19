use std::io::Read;

use crate::support::ByteSeq;

#[derive(Debug)]
pub struct ClassFile;

impl ClassFile {
    const MAGIC_NUMBER: u32 = 0xCAFEBABE;

    pub fn parse<R: Read>(
        input: &mut R,
    ) -> Result<ClassFile, Box<dyn std::error::Error + 'static>> {
        let mut bs = ByteSeq::new(input)?;

        // Check if it starts with magic number
        if bs.read_u32() != Self::MAGIC_NUMBER {
            Err("not a java class file")?;
        }
        // skip major and minor version
        bs.skip(4);

        Ok(ClassFile)
    }
}
