extern crate core;

use std::fs::File;
use std::path::Path;

mod components;
mod parser;
mod writer;

/// Parse an arsc file into structured data
///
/// # Argument:
/// * path - the path pointing to the arsc file
/// # Returns:
/// a parsed arsc struct
/// # Error:
/// * io errors
pub fn parse<P: AsRef<Path>>(path: P) -> std::io::Result<components::Arsc> {
    parser::parse(File::open(path)?)
}

/// Write a structured Arsc to arsc file
///
/// # Arguments:
/// * arsc - a structured Arsc file needs to be written
/// * output_path - the path pointing to the written out arsc file
///
/// # Returns:
/// the number of bytes that have been written
/// # Error:
/// * io errors
pub fn write<P: AsRef<Path>>(arsc: components::Arsc, output_path: P) -> std::io::Result<usize> {
    writer::write(arsc, output_path.as_ref())
}
