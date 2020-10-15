use byteorder::{WriteBytesExt, LE};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};

mod tag;
pub use tag::{MemMapEntry, RegionType, Tag, TagType};

pub fn bootinfo_size(tags: &[Tag]) -> u32 {
    8 + tags.iter().map(Tag::get_size).sum::<u32>()
}

pub fn write_bootinfo<F: Write + Seek>(tags: &[Tag], mut buf: F, offset: u64) -> Result<()> {
    if tags.last() != Some(&Tag::End) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "bootinfo tags must end with Tag::End",
        ));
    }

    buf.seek(SeekFrom::Start(offset))?;

    buf.write_u32::<LE>(bootinfo_size(tags))?;
    buf.write_u32::<LE>(0)?;

    for tag in tags {
        Tag::write_tag(tag, &mut buf)?;
    }
    Ok(())
}
