use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Result, Seek, SeekFrom};

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
            .wrapping_add(self.header_length)
            == 0
    }
}

// Defined by Multiboot2 spec
const SEARCH_END: usize = 32768;
const ALIGNMENT: usize = 8;

pub fn find_header<F: Read + Seek>(mut kernel_image: F) -> Result<Option<u64>> {
    for offset in (0..SEARCH_END).step_by(ALIGNMENT) {
        kernel_image.seek(SeekFrom::Start(offset as u64))?;

        let word = kernel_image.read_u32::<LE>()?;

        if word == HEADER_MAGIC {
            let header = Header {
                magic: word,
                architecture: kernel_image.read_u32::<LE>()?,
                header_length: kernel_image.read_u32::<LE>()?,
                checksum: kernel_image.read_u32::<LE>()?,
            };
            if header.is_valid() {
                return Ok(Some(offset as u64));
            }
        }
    }
    Ok(None)
}

mod tag;
pub use self::tag::{Tag, TagType};
mod iter;

pub fn iter_tags<F: Read + Seek>(kernel_image: F, offset: u64) -> Result<iter::TagIter> {
    iter::TagIter::new(kernel_image, offset)
}
