use std::mem::size_of;
use std::io::{Result, Write};
use byteorder::{WriteBytesExt, LE};

use num_enum::IntoPrimitive;

#[derive(Debug)]
pub struct MemMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    // 4 reserved bytes as padding
}

#[repr(u32)]
#[derive(IntoPrimitive)]
pub enum TagType {
    End = 0,
    BasicMeminfo = 4,
    MemMap = 6,

    #[cfg(feature = "hvm")]
    HybridRuntime = 0xF00DF00D,
}

#[repr(u32)]
#[derive(IntoPrimitive)]
pub enum RegionType {
    Available = 1,
    AcpiReclaimable = 3,
    NonVolatile = 4,
    Defective = 5,
}

#[derive(Debug)]
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
        #[allow(unused_variables)]
        match self {
            Tag::BasicMeminfo{mem_lower, mem_upper} => TagType::BasicMeminfo,
            Tag::MemMap{entries} => TagType::MemMap,
            #[cfg(feature = "hvm")]
            Tag::HybridRuntime{total_num_apics, first_hrt_apic_id, have_hrt_ioapic, first_hrt_ioapic_entry, cpu_freq_khz, hrt_flags, max_mem_mapped, first_hrt_gpa, boot_state_gpa, gva_offset, comm_page_gpa, hrt_int_vector} => TagType::HybridRuntime,
            Tag::End => TagType::End,
        }
    }

    pub fn get_size(&self) -> u32 {
        #[allow(unused_variables)]
        return 8 + match self {
            Tag::BasicMeminfo{mem_lower, mem_upper} => 8,
            Tag::MemMap{entries} => 8 + size_of::<MemMapEntry>() * entries.len(),
            #[cfg(feature = "hvm")]
            Tag::HybridRuntime{total_num_apics, first_hrt_apic_id, have_hrt_ioapic, first_hrt_ioapic_entry, cpu_freq_khz, hrt_flags, max_mem_mapped, first_hrt_gpa, boot_state_gpa, gva_offset, comm_page_gpa, hrt_int_vector} => 80,
            Tag::End => 0,
        } as u32
    }

    pub fn write_tag<F: Write>(&self, mut buf: F) -> Result<()> {
        buf.write_u32::<LE>(self.get_type().into())?;
        buf.write_u32::<LE>(self.get_size())?;

        match self {
            Tag::BasicMeminfo{mem_lower, mem_upper} => {
                buf.write_u32::<LE>(*mem_lower)?;
                buf.write_u32::<LE>(*mem_upper)?;
            },
            Tag::MemMap{entries} => {
                buf.write_u32::<LE>(size_of::<MemMapEntry>() as u32)?;
                buf.write_u32::<LE>(0)?; // entry_version
                for entry in entries {
                    buf.write_u64::<LE>(entry.base_addr)?;
                    buf.write_u64::<LE>(entry.length)?;
                    buf.write_u32::<LE>(entry.entry_type)?;
                    buf.write_u32::<LE>(0)?;
                }
            },
            #[cfg(feature = "hvm")]
            Tag::HybridRuntime {total_num_apics, first_hrt_apic_id, have_hrt_ioapic, first_hrt_ioapic_entry, cpu_freq_khz, hrt_flags, max_mem_mapped, first_hrt_gpa, boot_state_gpa, gva_offset, comm_page_gpa, hrt_int_vector} => {
                buf.write_u32::<LE>(*total_num_apics)?;
                buf.write_u32::<LE>(*first_hrt_apic_id)?;
                buf.write_u32::<LE>(*have_hrt_ioapic as u32)?;
                buf.write_u32::<LE>(*first_hrt_ioapic_entry)?;
                buf.write_u64::<LE>(*cpu_freq_khz)?;
                buf.write_u64::<LE>(*hrt_flags)?;
                buf.write_u64::<LE>(*max_mem_mapped)?;
                buf.write_u64::<LE>(*first_hrt_gpa)?;
                buf.write_u64::<LE>(*boot_state_gpa)?;
                buf.write_u64::<LE>(*gva_offset)?;
                buf.write_u64::<LE>(*comm_page_gpa)?;
                buf.write_u8(*hrt_int_vector)?;
                buf.write(&[0,0,0,0,0,0,0])?;
            },
            Tag::End => {},
        }
        Ok(())
    }
}
