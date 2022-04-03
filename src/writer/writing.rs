use super::components_writing::ArscSerializable;
use super::with_header::WithHeader;
use super::write_util;
use crate::components::Arsc;
use std::io::{BufWriter, Result, Write};

pub(in crate::writer) struct Writer<W: Write> {
    writer: BufWriter<W>,
    position: usize,
}

impl<W: Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Writer {
            writer: BufWriter::new(writer),
            position: 0,
        }
    }

    pub fn write(&mut self, arsc: &Arsc) -> Result<usize> {
        self.position += arsc.header().write(&mut self.writer)?;
        self.position += write_util::write_u32(&mut self.writer, arsc.packages.len())?;
        self.position += arsc.global_string_pool.write(&mut self.writer)?;

        Ok(self.position)
    }
}
