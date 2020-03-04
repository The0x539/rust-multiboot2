mod header;

fn main() -> std::io::Result<()> {
    let f = std::fs::File::open("nautilus_hrt.bin")?;
    for tag in header::iter_tags(f, 0x1000)? {
        println!("{:#X?}", tag);
        if let Ok(header::Tag::HybridRuntime(_,_,_,_,_,_)) = tag {
            println!("hrt");
        }
    }
    Ok(())
}
