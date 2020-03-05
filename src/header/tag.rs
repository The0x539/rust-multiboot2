#[allow(non_snake_case, non_upper_case_globals)] // this is basically an enum
pub mod TagType {
    pub const End: u16 = 0;
    pub const InfoRequest: u16 = 1;
    pub const LoadAddr: u16 = 2;
    pub const EntryAddr: u16 = 3;
    pub const ConsoleFlags: u16 = 4;
    pub const Framebuffer: u16 = 5;
    pub const ModuleAlign: u16 = 6;
    pub const EfiBootServices: u16 = 7;
    pub const EntryAddrEfi32: u16 = 8;
    pub const EntryAddrEfi64: u16 = 9;
    pub const Relocatable: u16 = 10;
    #[cfg(feature = "hvm")]
    pub const HybridRuntime: u16 = 0xF00D;
}

#[derive(Debug)]
pub enum Tag {
    End,
    InfoRequest (
        Vec<u32>, //mbi_tag_types
    ),
    LoadAddr (
        u32, //header_addr
        u32, //load_addr
        u32, //load_end_addr
        u32, //bss_end_addr
    ),
    EntryAddr (u32),
    EntryAddrEfi32 (u32),
    EntryAddrEfi64 (u32),
    ConsoleFlags (u32),
    Framebuffer (
        u32, //width
        u32, //height
        u32, //depth
    ),
    ModuleAlign,
    EfiBootServices,
    Relocatable (
        u32, //min_addr
        u32, //max_addr
        u32, //align
        u32, //preference
    ),
    #[cfg(feature = "hvm")]
    HybridRuntime (
        u64, //flags
        u64, //gpa_map_req
        u64, //hrt_hihalf_offset
        u64, //nautilus_entry_gva
        u64, //comm_page_gpa
        u64, //int_vec
    ),
    Unknown (
        u16, //type
        u16, //flags
        u32, //size
    ),
}
