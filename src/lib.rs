//! `arsc` is a simple library that enables parsing and writing Android resource files (arsc)
//!
//! # Example
//! ```rust,no_run
//! use arsc::{parse, write};
//!
//! fn main() -> std::io::Result<()> {
//!     let arsc = parse("/resources.arsc")?;
//!     let _ = write(&arsc, "/output.arsc")?;
//!     Ok(())
//! }
//! ```

extern crate core;

use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;

pub mod components;
mod parser;
mod writer;
pub use components::*;

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

/// Parse an arsc file into structured data
///
/// # Argument:
/// * reader - a seekable reader that reads bytes data from the arsc file
/// # Returns:
/// a parsed arsc struct
/// # Error:
/// * io errors
pub fn parse_from<R: Read + Seek>(reader: R) -> std::io::Result<components::Arsc> {
    parser::parse(reader)
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
pub fn write<P: AsRef<Path>>(arsc: &components::Arsc, output_path: P) -> std::io::Result<usize> {
    let mut writer = std::io::BufWriter::new(File::create(output_path)?);
    write_to(arsc, &mut writer)
}

/// Write a structured Arsc to designated writer
///
/// # Arguments:
/// * arsc - a structured Arsc file needs to be written
/// * output - the output writer that the bytes will be written to
///
/// # Returns:
/// the number of bytes that have been written
/// # Error:
/// * io errors
pub fn write_to<W: Write>(arsc: &components::Arsc, output: &mut W) -> std::io::Result<usize> {
    writer::write(arsc, output)
}
