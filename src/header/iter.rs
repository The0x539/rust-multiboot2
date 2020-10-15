use super::tag::{Tag, TagType};
use super::Header;
use byteorder::{ReadBytesExt, LE};

use std::convert::TryFrom;

use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};

#[derive(Debug)]
pub struct TagIter {
    done: bool,
    buf: Cursor<Vec<u8>>,
}
impl TagIter {
    pub fn new<F: Read + Seek>(mut kernel_image: F, offset: u64) -> Result<Self> {
        kernel_image.seek(SeekFrom::Start(offset))?;
        let header = Header {
            magic: kernel_image.read_u32::<LE>()?,
            architecture: kernel_image.read_u32::<LE>()?,
            header_length: kernel_image.read_u32::<LE>()?,
            checksum: kernel_image.read_u32::<LE>()?,
        };

        if !header.is_valid() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "No valid Multiboot2 header at offset",
            ));
        }

        let mut buf = vec![0u8; header.header_length as usize - 16];
        kernel_image.read_exact(&mut buf)?;

        Ok(TagIter {
            done: false,
            buf: Cursor::new(buf),
        })
    }

    fn next_tag(&mut self) -> Result<Tag> {
        if self.done {
            return Err(Error::new(ErrorKind::UnexpectedEof, "No more tags"));
        }

        let tag_type = self.buf.read_u16::<LE>()?;
        let tag_flags = self.buf.read_u16::<LE>()?;
        let tag_size = self.buf.read_u32::<LE>()?;

        let tag = match TagType::try_from(tag_type) {
            Ok(ty) => ty.read_fields(tag_size, &mut self.buf)?,
            Err(_) => Tag::Unknown(tag_type, tag_flags, tag_size),
        };

        self.buf.set_position((self.buf.position() + 7) & !7);

        Ok(tag)
    }
}
impl Iterator for TagIter {
    type Item = Result<Tag>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        Some(self.next_tag())
    }
}
