use super::write_util;
use crate::components::{Header, ResourceEntry, ResourceValue, StringPool, Value};
use crate::writer::components_sizing::padding;
use crate::writer::with_header::WithHeader;
use std::io::{Result, Write};

pub trait ArscSerializable {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize>;
}

impl ArscSerializable for Value {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut written = write_util::write_u16(output, self.size)?;
        written += write_util::write_u8(output, self.zero)?;
        written += write_util::write_u8(output, self.r#type)?;
        written += write_util::write_u32(output, self.r#type)?;
        Ok(written)
    }
}

impl ArscSerializable for ResourceValue {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        match self {
            ResourceValue::Plain(value) => value.write(output),
            ResourceValue::Bag { parent, values } => {
                let mut written = write_util::write_u32(output, *parent)?;
                written += write_util::write_u32(output, values.len())?;
                for (index, value) in values {
                    written += write_util::write_u32(output, *index)?;
                    written += value.write(output)?;
                }
                Ok(written)
            }
        }
    }
}

impl ArscSerializable for ResourceEntry {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut written = write_util::write_u16(output, self.flags)?;
        written += self.value.write(output)?;
        Ok(written)
    }
}

impl ArscSerializable for Header {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut written = write_util::write_u16(output, self.type_flag as u16)?;
        written += write_util::write_u16(output, self.header_size)?;
        written += write_util::write_u32(output, self.size)?;
        Ok(written)
    }
}

impl ArscSerializable for StringPool {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = self.header().write(output)?;
        position += write_util::write_u32(output, self.strings.len())?;
        position += write_util::write_u32(output, 0)?; // style_count
        position += write_util::write_u32(output, self.flags)?;
        position += write_util::write_u32(output, 7 * 4 + self.strings.len() * 4)?; // string_offset
        position += write_util::write_u32(output, 0)?; // style offset
        position += self.write_offsets(output)?;
        position += self.write_strings(output)?;
        position += output.write(&vec![0; padding(position)])?;

        Ok(position)
    }
}

impl StringPool {
    fn write_offsets<W: Write>(&self, output: &mut W) -> Result<usize> {
        if self.flags & StringPool::UTF8_FLAG != 0 {
            self.write_utf8_offsets(output)
        } else {
            self.write_utf16_offsets(output)
        }
    }

    fn write_utf8_offsets<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut offset = 0;
        let mut written = 0;
        for string in &self.strings {
            written += write_util::write_u32(output, offset)?;
            let char_count = string.chars().count();
            offset += if char_count > 0x7F { 2 } else { 1 };
            let byte_count = string.len();
            offset += if byte_count > 0x7F { 2 } else { 1 };
            offset += byte_count + 1; // 1 for null terminator
        }
        Ok(written)
    }

    fn write_utf16_offsets<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut offset = 0;
        let mut written = 0;
        for string in &self.strings {
            written += write_util::write_u32(output, offset)?;
            let char_count = string.encode_utf16().count();
            offset += if char_count > 0x7FFF { 4 } else { 2 };
            offset += char_count * 2 + 2; // 2 for null terminator
        }
        Ok(written)
    }

    fn write_strings<W: Write>(&self, buffer: &mut W) -> Result<usize> {
        let mut position = 0;
        for string in &self.strings {
            if self.flags & StringPool::UTF8_FLAG != 0 {
                let char_count = string.chars().count();
                position += Self::write_utf8_length(char_count, buffer)?;
                position += Self::write_utf8_length(string.len(), buffer)?;
                position += buffer.write(string.as_bytes())?;
                position += write_util::write_u8(buffer, 0)?;
            } else {
                position += Self::write_utf16_length(string.encode_utf16().count(), buffer)?;
                position += write_util::write_string_utf16(buffer, string)?;
                position += write_util::write_u16(buffer, 0)?;
            }
        }
        Ok(position)
    }

    fn write_utf8_length<W: Write>(length: usize, buffer: &mut W) -> Result<usize> {
        let mut offset = 0;
        if length > 0x7F {
            offset += write_util::write_u8(buffer, (length >> 8) | 0x80)?;
        }
        offset += write_util::write_u8(buffer, length & 0x7F)?;
        Ok(offset)
    }

    fn write_utf16_length<W: Write>(length: usize, buffer: &mut W) -> Result<usize> {
        let mut offset = 0;
        if length > 0x7FFF {
            let leading_two_bytes = (length >> 16) | 0x8000;
            offset += write_util::write_u8(buffer, leading_two_bytes & 0x7F)?;
            offset += write_util::write_u8(buffer, leading_two_bytes >> 8)?;
        }
        offset += write_util::write_u8(buffer, length & 0x7F)?;
        offset += write_util::write_u8(buffer, (length >> 8) & 0x7F)?;
        Ok(offset)
    }
}
