use std::convert::TryFrom;

use byteorder::{ReadBytesExt, LE};
use num_enum::TryFromPrimitive;
use thiserror::Error;

#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive)]
pub enum TagType {
    End = 0,
    InfoRequest = 1,
    LoadAddr = 2,
    EntryAddr = 3,
    ConsoleFlags = 4,
    Framebuffer = 5,
    ModuleAlign = 6,
    EfiBootServices = 7,
    EntryAddrEfi32 = 8,
    EntryAddrEfi64 = 9,
    Relocatable = 10,

    #[cfg(feature = "hvm")]
    HybridRuntime = 0xF00D,
}

impl TagType {
    fn read_fields<R: std::io::Read>(&self, size: u32, mut r: R) -> std::io::Result<Tag> {
        let mut get_u32 = || r.read_u32::<LE>();

        let tag = match self {
            Self::End => Tag::End,
            Self::InfoRequest => {
                let num_requests = (size as usize - 8) / 4;
                let mut mbi_tag_types = vec![0u32; num_requests];
                r.read_u32_into::<LE>(&mut mbi_tag_types)?;
                Tag::InfoRequest { mbi_tag_types }
            }
            Self::LoadAddr => Tag::LoadAddr {
                header_addr: get_u32()?,
                load_addr: get_u32()?,
                load_end_addr: get_u32()?,
                bss_end_addr: get_u32()?,
            },
            Self::EntryAddr => Tag::EntryAddr(get_u32()?),
            Self::EntryAddrEfi32 => Tag::EntryAddrEfi32(get_u32()?),
            Self::EntryAddrEfi64 => Tag::EntryAddrEfi64(get_u32()?),
            Self::ConsoleFlags => Tag::ConsoleFlags(get_u32()?),
            Self::Framebuffer => Tag::Framebuffer {
                width: get_u32()?,
                height: get_u32()?,
                depth: get_u32()?,
            },
            Self::ModuleAlign => Tag::ModuleAlign,
            Self::EfiBootServices => Tag::EfiBootServices,
            Self::Relocatable => Tag::Relocatable {
                min_addr: get_u32()?,
                max_addr: get_u32()?,
                align: get_u32()?,
                preference: get_u32()?,
            },
            #[cfg(feature = "hvm")]
            Self::HybridRuntime => {
                let mut fields = [0u64; 6];
                r.read_u64_into::<LE>(&mut fields)?;
                Tag::HybridRuntime {
                    flags: fields[0],
                    gpa_map_req: fields[1],
                    hrt_hihalf_offset: fields[2],
                    nautilus_entry_gva: fields[3],
                    comm_page_gpa: fields[4],
                    int_vec: fields[5],
                }
            }
        };

        match size.checked_sub(tag.size()) {
            Some(0) => (),
            Some(n @ 1..=7) => {
                // Padding bytes; just read past them
                let mut dummy = [0; 7];
                r.read_exact(&mut dummy[0..n as usize])?;
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unexpected tag size: expected {}, got {}", tag.size(), size),
                ))
            }
        }

        Ok(tag)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    End,
    InfoRequest {
        mbi_tag_types: Vec<u32>,
    },
    LoadAddr {
        header_addr: u32,
        load_addr: u32,
        load_end_addr: u32,
        bss_end_addr: u32,
    },
    EntryAddr(u32),
    EntryAddrEfi32(u32),
    EntryAddrEfi64(u32),
    ConsoleFlags(u32),
    Framebuffer {
        width: u32,
        height: u32,
        depth: u32,
    },
    ModuleAlign,
    EfiBootServices,
    Relocatable {
        min_addr: u32,
        max_addr: u32,
        align: u32,
        preference: u32,
    },
    #[cfg(feature = "hvm")]
    HybridRuntime {
        flags: u64,
        gpa_map_req: u64,
        hrt_hihalf_offset: u64,
        nautilus_entry_gva: u64,
        comm_page_gpa: u64,
        int_vec: u64,
    },
}

#[derive(Error, Debug)]
#[error("{self:?}")]
pub struct UnknownTag {
    tag_type: u16,
    flags: u16,
    size: u32,
    data: Vec<u8>,
}

impl Tag {
    pub fn tag_type(&self) -> TagType {
        match self {
            Self::End => TagType::End,
            Self::InfoRequest { .. } => TagType::InfoRequest,
            Self::LoadAddr { .. } => TagType::LoadAddr,
            Self::EntryAddr { .. } => TagType::EntryAddr,
            Self::EntryAddrEfi32(..) => TagType::EntryAddrEfi32,
            Self::EntryAddrEfi64(..) => TagType::EntryAddrEfi64,
            Self::ConsoleFlags(..) => TagType::ConsoleFlags,
            Self::Framebuffer { .. } => TagType::Framebuffer,
            Self::ModuleAlign => TagType::ModuleAlign,
            Self::EfiBootServices => TagType::EfiBootServices,
            Self::Relocatable { .. } => TagType::Relocatable,
            #[cfg(feature = "hvm")]
            Self::HybridRuntime { .. } => TagType::HybridRuntime,
        }
    }

    pub fn size(&self) -> u32 {
        match self {
            Self::End | Self::ModuleAlign | Self::EfiBootServices => 8,
            Self::EntryAddr(..)
            | Self::EntryAddrEfi32(..)
            | Self::EntryAddrEfi64(..)
            | Self::ConsoleFlags(..) => 12,
            Self::Framebuffer { .. } => 20,
            Self::LoadAddr { .. } | Self::Relocatable { .. } => 24,
            #[cfg(feature = "hvm")]
            Self::HybridRuntime { .. } => 56,
            Self::InfoRequest { mbi_tag_types } => 4 * mbi_tag_types.len() as u32 + 8,
        }
    }

    pub fn from_reader<R: std::io::Read>(mut r: R) -> std::io::Result<Self> {
        let ty = r.read_u16::<LE>()?;
        let flags = r.read_u16::<LE>()?;
        let size = r.read_u32::<LE>()?;

        let tag = if let Ok(tag_type) = TagType::try_from(ty) {
            tag_type.read_fields(size, &mut r)?
        } else {
            let mut data = vec![0u8; size as usize - 8];
            r.read_exact(&mut data)?;
            let unknown_tag = UnknownTag {
                tag_type: ty,
                flags,
                size,
                data,
            };
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                unknown_tag,
            ));
        };

        Ok(tag)
    }
}
