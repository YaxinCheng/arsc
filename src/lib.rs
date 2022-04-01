extern crate core;

use std::fs::File;
use std::path::Path;
use crate::components::Header;

mod parser;
mod writer;
mod components;

pub fn parse<P: AsRef<Path>>(path: P) {
    let file = File::open(path).expect("File not opened");
    parser::parse(file);
}