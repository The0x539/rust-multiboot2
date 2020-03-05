extern crate vm_memory;

use std::mem::size_of;

pub struct MemMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    reserved: u32,
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
    fn encode(&self) -> Vec<u8> {
        let size = 8 + match self {
            Tag::BasicMeminfo{mem_lower: _, mem_upper: _} => 8,
            Tag::MemMap{entries} => size_of::<MemMapEntry>() * entries.len(),
            Tag::HybridRuntime {
                total_num_apics: _,
                first_hrt_apic_id: _,
                have_hrt_ioapic: _,
                first_hrt_ioapic_entry: _,
                cpu_freq_khz: _,
                hrt_flags: _,
                max_mem_mapped: _,
                first_hrt_gpa: _,
                boot_state_gpa: _,
                gva_offset: _,
            } => 4 + 4 + 4 + 4 + 8 + 8 + 8 + 8 + 8 + 8,
            Tag::End => 0,
        };
        let buf: Vec<u8> = vec![0; size];
        buf
    }
}
