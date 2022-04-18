use crate::components::Arsc;
use components_writing::ArscSerializable;
use std::io::{Result, Write};

mod components_sizing;
mod components_writing;
mod with_header;
mod write_util;

pub fn write<W: Write>(arsc: &Arsc, output: &mut W) -> Result<usize> {
    arsc.write(output)
}
