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

        let mut buf: Vec<u8> = vec![0; header.header_length as usize - 16];
        kernel_image.read_exact(&mut buf)?;

        Ok(TagIter {
            done: false,
            buf: Cursor::new(buf),
        })
    }

    fn u16(&mut self) -> Result<u16> {
        self.buf.read_u16::<LE>()
    }
    fn u32(&mut self) -> Result<u32> {
        self.buf.read_u32::<LE>()
    }
    #[allow(dead_code)]
    fn u64(&mut self) -> Result<u64> {
        self.buf.read_u64::<LE>()
    }

    fn next_tag(&mut self) -> Result<Tag> {
        macro_rules! tag {
            ($tag:ident: $($memb:ident$(,)?)+) => {
                Tag::$tag($(self.$memb()?),+)
            };

            ($tag:ident) => {
                Tag::$tag
            };
        }

        if self.done {
            return Err(Error::new(ErrorKind::UnexpectedEof, "No more tags"));
        }

        let tag_type = self.u16()?;
        let tag_flags = self.u16()?;
        let tag_size = self.u32()?;

        let tag = if let Ok(ty) = TagType::try_from(tag_type) {
            match ty {
                TagType::End => {
                    self.done = true;
                    Tag::End
                }

                TagType::InfoRequest => {
                    let mut mbi_tag_types: Vec<u32> = vec![0; (tag_size as usize - 8) / 4];
                    self.buf.read_u32_into::<LE>(&mut mbi_tag_types)?;
                    Tag::InfoRequest(mbi_tag_types)
                }

                TagType::LoadAddr => tag!(LoadAddr: u32, u32, u32, u32),
                TagType::EntryAddr => tag!(EntryAddr: u32),
                TagType::EntryAddrEfi32 => tag!(EntryAddrEfi32: u32),
                TagType::EntryAddrEfi64 => tag!(EntryAddrEfi64: u32),
                TagType::ConsoleFlags => tag!(ConsoleFlags: u32),
                TagType::Framebuffer => tag!(Framebuffer: u32, u32, u32),
                TagType::ModuleAlign => tag!(ModuleAlign),
                TagType::EfiBootServices => tag!(EfiBootServices),
                TagType::Relocatable => tag!(Relocatable: u32, u32, u32, u32),

                #[cfg(feature = "hvm")]
                TagType::HybridRuntime => tag!(HybridRuntime: u64, u64, u64, u64, u64, u64),
            }
        } else {
            Tag::Unknown(tag_type, tag_flags, tag_size)
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
