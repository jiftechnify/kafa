#![allow(dead_code)]

use std::io::{Read, Result};

pub struct ByteSeq {
    buf: Vec<u8>,
    i: usize,
}

impl ByteSeq {
    pub fn new<R: Read>(mut r: R) -> Result<Self> {
        let mut buf = Vec::new();
        r.read_to_end(&mut buf)?;
        Ok(ByteSeq { buf, i: 0 })
    }

    pub fn skip(&mut self, n: usize) {
        self.i += n;
    }

    pub fn seek(&mut self, pos: usize) {
        self.i = pos;
    }

    pub fn pos(&self) -> usize {
        self.i
    }

    pub fn read_u8(&mut self) -> u8 {
        let b = self.buf[self.i];
        self.i += 1;
        b
    }

    pub fn read_u16(&mut self) -> u16 {
        let n = u16::from_be_bytes(self.buf[self.i..self.i + 2].try_into().unwrap());
        self.i += 2;
        n
    }

    pub fn read_u32(&mut self) -> u32 {
        let n = u32::from_be_bytes(self.buf[self.i..self.i + 4].try_into().unwrap());
        self.i += 4;
        n
    }

    pub fn read_bytes(&mut self, n: usize) -> Vec<u8> {
        let i = self.i;
        let bytes = self.buf[i..i + n].to_vec();
        self.i += n;
        bytes
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_byte_seq() {
        let mut bin = Cursor::new(vec![255u8, 1, 0, 0, 1, 0, 0, 0]);
        let mut bs = ByteSeq::new(&mut bin).unwrap();

        assert_eq!(bs.pos(), 0);

        assert_eq!(bs.read_u8(), 255);
        assert_eq!(bs.pos(), 1);

        // [1, 0]
        assert_eq!(bs.read_u16(), 256);
        assert_eq!(bs.pos(), 3);

        bs.skip(1);
        assert_eq!(bs.pos(), 4);

        // [1, 0, 0, 0]
        assert_eq!(bs.read_u32(), 16777216);
        assert_eq!(bs.pos(), 8);

        bs.seek(0);
        assert_eq!(bs.pos(), 0);
        assert_eq!(bs.read_bytes(8), vec![255, 1, 0, 0, 1, 0, 0, 0]);
        assert_eq!(bs.pos(), 8);
    }
}
