mod read_util;
mod components_read;
mod parsing;

use std::io::{Read, Seek};

pub fn parse<R: Read + Seek>(reader: R) {
    parsing::Parser::new(reader).parse().expect("");
}