use byteorder::{ReadBytesExt, LE};
use std::io::{Read, Seek, SeekFrom};

mod tag;
pub use tag::{Tag, TagType};

// Defined by Multiboot2 spec
pub const HEADER_MAGIC: u32 = 0xE85250D6;
pub const SEARCH_END: u64 = 32768;
pub const ALIGNMENT: usize = 8;

#[derive(Debug, Copy, Clone)]
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

pub fn find_header<R: Read + Seek>(mut image: R) -> std::io::Result<Option<(u64, Header)>> {
    for offset in (0..SEARCH_END).step_by(ALIGNMENT) {
        image.seek(SeekFrom::Start(offset))?;

        let magic = image.read_u32::<LE>()?;
        if magic == HEADER_MAGIC {
            let header = Header {
                magic,
                architecture: image.read_u32::<LE>()?,
                header_length: image.read_u32::<LE>()?,
                checksum: image.read_u32::<LE>()?,
            };
            if header.is_valid() {
                return Ok(Some((offset, header)));
            }
        }
    }
    Ok(None)
}

#[derive(Debug, Clone)]
pub struct TagIter<R> {
    done: bool,
    data: R,
}

impl<R: Read> TagIter<R> {
    pub fn new(data: R) -> Self {
        Self { done: false, data }
    }
}

impl<R: Read> Iterator for TagIter<R> {
    type Item = std::io::Result<Tag>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let tag = match Tag::from_reader(&mut self.data) {
            Ok(t) => t,
            Err(e) => return Some(Err(e)),
        };
        if tag == Tag::End {
            self.done = true;
        }

        Some(Ok(tag))
    }
}
