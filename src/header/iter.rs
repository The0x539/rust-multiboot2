use std::io::Read;

use super::tag::Tag;

#[derive(Debug, Clone)]
pub struct TagIter<R> {
    done: bool,
    data: R,
}

impl<R: Read> TagIter<R> {
    pub fn new(data: R) -> Self {
        Self { done: false, data }
    }
}

impl<R: Read> Iterator for TagIter<R> {
    type Item = std::io::Result<Tag>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let tag = match Tag::from_reader(&mut self.data) {
            Ok(t) => t,
            Err(e) => return Some(Err(e)),
        };
        if tag == Tag::End {
            self.done = true;
        }

        Some(Ok(tag))
    }
}
