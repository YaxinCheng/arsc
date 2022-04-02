extern crate core;

use std::fs::File;
use std::path::Path;

mod components;
mod parser;
mod writer;

pub fn parse<P: AsRef<Path>>(path: P) -> std::io::Result<components::Arsc> {
    let file = File::open(path).expect("File not opened");
    parser::parse(file)
}

pub fn write(arsc: components::Arsc) {
    writer::write(arsc);
}
