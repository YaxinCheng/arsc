use crate::components::Arsc;
use components_writing::ArscSerializable;
use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::Path;

mod components_sizing;
mod components_writing;
mod with_header;
mod write_util;

pub fn write(arsc: Arsc, outpath: &Path) -> Result<usize> {
    let mut writer = BufWriter::new(File::create(outpath)?);
    arsc.write(&mut writer)
}
