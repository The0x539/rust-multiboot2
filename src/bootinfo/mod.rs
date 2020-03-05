use std::mem::size_of;
use std::io::Write;
use byteorder::{WriteBytesExt, LE};

pub struct MemMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    reserved: u32,
}

#[allow(non_snake_case, non_upper_case_globals)]
pub mod TagType {
    pub const End: u32 = 0;
    pub const BasicMeminfo: u32 = 4;
    pub const MemMap: u32 = 6;
    pub const HybridRuntime: u32 = 0xF00DF00D;
}

pub enum Tag {
    BasicMeminfo {
        mem_lower: u32,
        mem_upper: u32,
    },
    MemMap {
        entries: Vec<MemMapEntry>,
    },
    HybridRuntime {
        total_num_apics: u32,
        first_hrt_apic_id: u32,
        have_hrt_ioapic: u32,
        first_hrt_ioapic_entry: u32,
        cpu_freq_khz: u64,
        hrt_flags: u64,
        max_mem_mapped: u64,
        first_hrt_gpa: u64,
        boot_state_gpa: u64,
        gva_offset: u64,
    },
    End,
}

impl Tag {
    fn get_type(&self) -> u32 {
        match self {
            Tag::BasicMeminfo{mem_lower: _, mem_upper: _} => TagType::BasicMeminfo,
            Tag::MemMap{entries: _} => TagType::MemMap,
            Tag::HybridRuntime{total_num_apics: _, first_hrt_apic_id: _, have_hrt_ioapic: _, first_hrt_ioapic_entry: _, cpu_freq_khz: _, hrt_flags: _, max_mem_mapped: _, first_hrt_gpa: _, boot_state_gpa: _, gva_offset: _,} => TagType::HybridRuntime,
            Tag::End => TagType::End,
        }
    }

    fn get_size(&self) -> u32 {
        8 + match self {
            Tag::BasicMeminfo{mem_lower: _, mem_upper: _} => 8,
            Tag::MemMap{entries} => size_of::<MemMapEntry>() * entries.len(),
            Tag::HybridRuntime{total_num_apics: _, first_hrt_apic_id: _, have_hrt_ioapic: _, first_hrt_ioapic_entry: _, cpu_freq_khz: _, hrt_flags: _, max_mem_mapped: _, first_hrt_gpa: _, boot_state_gpa: _, gva_offset: _} => 4 + 4 + 4 + 4 + 8 + 8 + 8 + 8 + 8 + 8,
            Tag::End => 0,
        } as u32
    }

    fn write_to<F: Write>(&self, mut buf: F) -> std::io::Result<()> {
        buf.write_u32::<LE>(self.get_type())?;
        buf.write_u32::<LE>(self.get_size())?;

        match self {
            Tag::BasicMeminfo{mem_lower, mem_upper} => {
                buf.write_u32::<LE>(*mem_lower)?;
                buf.write_u32::<LE>(*mem_upper)?;
            },
            Tag::MemMap{entries} => {
                for entry in entries {
                    buf.write_u64::<LE>(entry.base_addr)?;
                    buf.write_u64::<LE>(entry.length)?;
                    buf.write_u32::<LE>(entry.entry_type)?;
                    buf.write_u32::<LE>(0)?;
                }
            },
            Tag::HybridRuntime {total_num_apics, first_hrt_apic_id, have_hrt_ioapic, first_hrt_ioapic_entry, cpu_freq_khz, hrt_flags, max_mem_mapped, first_hrt_gpa, boot_state_gpa, gva_offset} => {
                buf.write_u32::<LE>(*total_num_apics)?;
                buf.write_u32::<LE>(*first_hrt_apic_id)?;
                buf.write_u32::<LE>(*have_hrt_ioapic)?;
                buf.write_u32::<LE>(*first_hrt_ioapic_entry)?;
                buf.write_u64::<LE>(*cpu_freq_khz)?;
                buf.write_u64::<LE>(*hrt_flags)?;
                buf.write_u64::<LE>(*max_mem_mapped)?;
                buf.write_u64::<LE>(*first_hrt_gpa)?;
                buf.write_u64::<LE>(*boot_state_gpa)?;
                buf.write_u64::<LE>(*gva_offset)?;
            },
            Tag::End => {},
        }
        Ok(())
    }
}
