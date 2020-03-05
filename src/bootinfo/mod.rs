use std::mem::size_of;
use std::io::{self, Write, Seek, SeekFrom};
use byteorder::{WriteBytesExt, LE};

#[derive(Debug)]
pub struct MemMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub entry_type: u32,
    // 4 reserved bytes as padding
}

#[allow(non_snake_case, non_upper_case_globals)]
pub mod TagType {
    pub const End: u32 = 0;
    pub const BasicMeminfo: u32 = 4;
    pub const MemMap: u32 = 6;
    pub const HybridRuntime: u32 = 0xF00DF00D;
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
    fn get_type(&self) -> u32 {
        #[allow(unused_variables)]
        match self {
            Tag::BasicMeminfo{mem_lower, mem_upper} => TagType::BasicMeminfo,
            Tag::MemMap{entries} => TagType::MemMap,
            Tag::HybridRuntime{total_num_apics, first_hrt_apic_id, have_hrt_ioapic, first_hrt_ioapic_entry, cpu_freq_khz, hrt_flags, max_mem_mapped, first_hrt_gpa, boot_state_gpa, gva_offset, comm_page_gpa, hrt_int_vector} => TagType::HybridRuntime,
            Tag::End => TagType::End,
        }
    }

    fn get_size(&self) -> u32 {
        #[allow(unused_variables)]
        return 8 + match self {
            Tag::BasicMeminfo{mem_lower, mem_upper} => 8,
            Tag::MemMap{entries} => size_of::<MemMapEntry>() * entries.len(),
            Tag::HybridRuntime{total_num_apics, first_hrt_apic_id, have_hrt_ioapic, first_hrt_ioapic_entry, cpu_freq_khz, hrt_flags, max_mem_mapped, first_hrt_gpa, boot_state_gpa, gva_offset, comm_page_gpa, hrt_int_vector} => 80,
            Tag::End => 0,
        } as u32
    }

    fn write_tag<F: Write>(&self, mut buf: F) -> io::Result<()> {
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

pub fn bootinfo_size(tags: &[Tag]) -> u32 {
    // for some reason, rust is complaining about adding 8 to the list comp
    // so we're doing it the old fashionwed way
    let mut sum: u32 = 8;
    for tag in tags {
        sum += tag.get_size();
    }
    sum
}

pub fn write_bootinfo<F: Write + Seek>(
    tags: &[Tag],
    mut buf: F,
    offset: u64,
) -> io::Result<()> {
    if let Some(Tag::End) = tags.last() {} else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "bootinfo tags must end with Tag::End"));
    }

    buf.seek(SeekFrom::Start(offset))?;

    buf.write_u32::<LE>(bootinfo_size(tags))?;
    buf.write_u32::<LE>(0)?;

    for tag in tags {
        Tag::write_tag(tag, &mut buf)?;
    }
    Ok(())
}
