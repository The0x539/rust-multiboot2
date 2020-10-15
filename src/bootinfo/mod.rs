use byteorder::{WriteBytesExt, LE};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};

mod tag;
pub use tag::{MemMapEntry, RegionType, Tag, TagType};

pub fn bootinfo_size(tags: &[Tag]) -> u32 {
    // for some reason, rust is complaining about adding 8 to the list comp
    // so we're doing it the old fashionwed way
    let mut sum: u32 = 8;
    for tag in tags {
        sum += tag.get_size();
    }
    sum
}

pub fn write_bootinfo<F: Write + Seek>(tags: &[Tag], mut buf: F, offset: u64) -> Result<()> {
    if let Some(Tag::End) = tags.last() {
    } else {
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
