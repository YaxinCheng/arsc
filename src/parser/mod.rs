mod components_read;
mod parsing;
mod read_util;

use crate::components::Arsc;
use std::io::{Read, Result, Seek};

pub fn parse<R: Read + Seek>(reader: R) -> Result<Arsc> {
    parsing::Parser::new(reader).parse()
}
