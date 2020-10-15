use byteorder::{WriteBytesExt, LE};
use std::io::{Result, Write};
use std::mem::size_of;

use num_enum::IntoPrimitive;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MemMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    // 4 reserved bytes as padding
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive)]
pub enum TagType {
    End = 0,
    BasicMeminfo = 4,
    MemMap = 6,

    #[cfg(feature = "hvm")]
    HybridRuntime = 0xF00DF00D,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive)]
pub enum RegionType {
    Available = 1,
    AcpiReclaimable = 3,
    NonVolatile = 4,
    Defective = 5,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    BasicMeminfo {
        mem_lower: u32,
        mem_upper: u32,
    },
    MemMap {
        entries: Vec<MemMapEntry>,
    },
    #[cfg(feature = "hvm")]
    HybridRuntime {
        total_num_apics: u32,
        first_hrt_apic_id: u32,
        have_hrt_ioapic: bool, //4 bytes
        first_hrt_ioapic_entry: u32,
        cpu_freq_khz: u64,
        hrt_flags: u64,
        max_mem_mapped: u64,
        first_hrt_gpa: u64,
        boot_state_gpa: u64,
        gva_offset: u64,
        comm_page_gpa: u64,
        hrt_int_vector: u8,
        // 7 reserved bytes as padding
    },
    End,
}

impl Tag {
    pub fn get_type(&self) -> TagType {
        match self {
            Tag::BasicMeminfo { .. } => TagType::BasicMeminfo,
            Tag::MemMap { .. } => TagType::MemMap,
            #[cfg(feature = "hvm")]
            Tag::HybridRuntime { .. } => TagType::HybridRuntime,
            Tag::End => TagType::End,
        }
    }

    pub fn get_size(&self) -> u32 {
        let body_size = match self {
            Tag::BasicMeminfo { .. } => 8,
            Tag::MemMap { entries } => 8 + (size_of::<MemMapEntry>() * entries.len()) as u32,
            #[cfg(feature = "hvm")]
            Tag::HybridRuntime { .. } => 80,
            Tag::End => 0,
        };
        body_size + 8
    }

    pub fn write_tag<F: Write>(&self, mut buf: F) -> Result<()> {
        buf.write_u32::<LE>(self.get_type().into())?;
        buf.write_u32::<LE>(self.get_size())?;

        match self {
            Tag::BasicMeminfo {
                mem_lower,
                mem_upper,
            } => {
                buf.write_u32::<LE>(*mem_lower)?;
                buf.write_u32::<LE>(*mem_upper)?;
            }
            Tag::MemMap { entries } => {
                buf.write_u32::<LE>(size_of::<MemMapEntry>() as u32)?; // entry_size
                buf.write_u32::<LE>(0)?; // entry_version
                for entry in entries {
                    buf.write_u64::<LE>(entry.base_addr)?;
                    buf.write_u64::<LE>(entry.length)?;
                    buf.write_u32::<LE>(entry.entry_type)?;
                    buf.write_u32::<LE>(0)?;
                }
            }
            #[cfg(feature = "hvm")]
            Tag::HybridRuntime {
                total_num_apics,
                first_hrt_apic_id,
                have_hrt_ioapic,
                first_hrt_ioapic_entry,
                cpu_freq_khz,
                hrt_flags,
                max_mem_mapped,
                first_hrt_gpa,
                boot_state_gpa,
                gva_offset,
                comm_page_gpa,
                hrt_int_vector,
            } => {
                for x in &[
                    *total_num_apics,
                    *first_hrt_apic_id,
                    *have_hrt_ioapic as u32,
                    *first_hrt_ioapic_entry,
                ] {
                    buf.write_u32::<LE>(*x)?;
                }
                for x in &[
                    *cpu_freq_khz,
                    *hrt_flags,
                    *max_mem_mapped,
                    *first_hrt_gpa,
                    *boot_state_gpa,
                    *gva_offset,
                    *comm_page_gpa,
                ] {
                    buf.write_u64::<LE>(*x)?;
                }
                buf.write_u8(*hrt_int_vector)?;
                buf.write(&[0u8; 7])?;
            }
            Tag::End => {}
        }
        Ok(())
    }
}
