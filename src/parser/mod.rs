mod components_read;
mod read_util;

use crate::components::Arsc;
use std::io::{BufReader, Read, Result, Seek};

pub fn parse<R: Read + Seek>(reader: R) -> Result<Arsc> {
    let mut reader = BufReader::new(reader);
    Arsc::try_from(&mut reader)
}
