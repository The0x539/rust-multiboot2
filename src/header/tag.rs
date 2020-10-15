use num_enum::TryFromPrimitive;

#[repr(u16)]
#[derive(TryFromPrimitive)]
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

#[derive(Debug)]
pub enum Tag {
    End,
    InfoRequest(
        Vec<u32>, //mbi_tag_types
    ),
    LoadAddr(
        u32, //header_addr
        u32, //load_addr
        u32, //load_end_addr
        u32, //bss_end_addr
    ),
    EntryAddr(u32),
    EntryAddrEfi32(u32),
    EntryAddrEfi64(u32),
    ConsoleFlags(u32),
    Framebuffer(
        u32, //width
        u32, //height
        u32, //depth
    ),
    ModuleAlign,
    EfiBootServices,
    Relocatable(
        u32, //min_addr
        u32, //max_addr
        u32, //align
        u32, //preference
    ),
    #[cfg(feature = "hvm")]
    HybridRuntime(
        u64, //flags
        u64, //gpa_map_req
        u64, //hrt_hihalf_offset
        u64, //nautilus_entry_gva
        u64, //comm_page_gpa
        u64, //int_vec
    ),
    Unknown(
        u16, //type
        u16, //flags
        u32, //size
    ),
}
