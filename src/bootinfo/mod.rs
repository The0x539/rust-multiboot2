use byteorder::{WriteBytesExt, LE};
use std::io::Write;

mod tag;
pub use tag::{MemMapEntry, RegionType, Tag, TagType};

pub fn bootinfo_size<'a, I: IntoIterator<Item = &'a Tag>>(tags: I) -> u32 {
    8 + tags.into_iter().map(Tag::size).sum::<u32>()
}

pub fn write_bootinfo<'a, I, W>(tags: I, mut w: W) -> std::io::Result<()>
where
    // &'a [Tag] or &'a Vec<Tag> or std::slice::Iter<'a, Tag> or something
    I: IntoIterator<Item = &'a Tag> + Clone,
    W: Write,
{
    w.write_u32::<LE>(bootinfo_size(tags.clone()))?;
    w.write_u32::<LE>(0)?;

    let mut final_tag = None;
    for tag in tags {
        final_tag = Some(tag);
        tag.write_to(&mut w)?;
    }

    match final_tag {
        Some(Tag::End) => Ok(()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bootinfo tags must end with Tag::End",
        )),
    }
}
