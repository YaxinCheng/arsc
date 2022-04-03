use crate::components::Arsc;
use components_writing::ArscSerializable;
use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::Path;

mod components_sizing;
mod components_writing;
mod with_header;
mod write_util;

pub fn write(arsc: Arsc, output_path: &Path) -> Result<usize> {
    let mut writer = BufWriter::new(File::create(output_path)?);
    arsc.write(&mut writer)
}
