use crate::components::Arsc;
use crate::writer::writing::Writer;

mod components_sizing;
mod components_writing;
mod with_header;
mod write_util;
mod writing;

pub fn write(arsc: Arsc) {
    let size = arsc.packages[0].key_names.size();
    println!("{size}")
}
