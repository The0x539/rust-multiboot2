mod header;
mod bootinfo;

fn main() -> std::io::Result<()> {
    let f = std::fs::File::open("nautilus_hrt.bin")?;
    for tag in header::iter_tags(f, 0x1000)? {
        println!("{:#X?}", tag);
    }

    let bootinfo_tags = [
        #[cfg(feature = "hvm")]
        bootinfo::Tag::HybridRuntime {
            total_num_apics: 1,
            first_hrt_apic_id: 0,
            have_hrt_ioapic: false,
            first_hrt_ioapic_entry: 0,
            cpu_freq_khz: 1024,
            first_hrt_gpa: 0x0,
            hrt_flags: 0x0,
            max_mem_mapped: 0,
            boot_state_gpa: 0,
            gva_offset: 0xFFFF_8000_0000_0000,
            comm_page_gpa: 0x0,
            hrt_int_vector: 0x0,
        },
        bootinfo::Tag::BasicMeminfo {
            mem_lower: 640, //thank you, bill gates
            mem_upper: 1024 * 1024 * 1024 * 3, // 3 gigs?
        },
        bootinfo::Tag::MemMap {
            entries: vec![],
        },
        bootinfo::Tag::End
    ];

    println!("{:?}", bootinfo_tags);

    let mut buf = vec![0; bootinfo::bootinfo_size(&bootinfo_tags) as usize];

    bootinfo::write_bootinfo(&bootinfo_tags, std::io::Cursor::new(&mut buf), 0)?;
    
    println!("{:?}", buf);

    Ok(())
}
