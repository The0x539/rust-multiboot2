extern crate byteorder;

use std::io::{
    self,
    Read,
    Seek,
};

pub const HEADER_MAGIC: u32 = 0xE85250D6;

#[derive(Debug)]
pub struct Header {
    pub magic: u32,
    pub architecture: u32,
    pub header_length: u32,
    pub checksum: u32,
}
impl Header {
    pub fn is_valid(&self) -> bool {
        if self.magic != HEADER_MAGIC {
            return false;
        }
        self.checksum
            .wrapping_add(self.magic)
            .wrapping_add(self.architecture)
            .wrapping_add(self.header_length) == 0
    }
}

mod tag;
pub use self::tag::Tag;
mod iter;

pub fn iter_tags<F: Read + Seek>(
    kernel_image: F,
    offset: u64,
) -> io::Result<iter::TagIter> {
    iter::TagIter::new(kernel_image, offset)
}
