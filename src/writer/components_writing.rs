use super::write_util;
use crate::components::{
    Arsc, Config, Header, Package, ResourceEntry, ResourceValue, Spec, Specs, StringPool, Type,
    Value,
};
use crate::writer::components_sizing::{padding, ByteSizing, ConstByteSizing};
use crate::writer::with_header::WithHeader;
use crate::{Resources, Style, StyleSpan};
use std::io::{Result, Write};

/// types that implement this trait should define the function
/// `write` to serialize and write the serialized bytes to the output
pub(crate) trait ArscSerializable {
    /// Serialize and write bytes to output
    /// # Argument:
    /// * output - a writer, where the bytes should be written to
    /// # Returns:
    /// the number of bytes that have been written out
    /// # Errors:
    /// io errors
    fn write<W: Write>(&self, output: &mut W) -> Result<usize>;
}

impl ArscSerializable for Value {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut written = write_util::write_u16(output, self.size)?;
        written += write_util::write_u8(output, self.zero)?;
        written += write_util::write_u8(output, self.r#type)?;
        written += write_util::write_u32(output, self.data_index)?;
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
        let size = if self.is_bag() { 16 } else { 8 };
        let mut written = write_util::write_u16(output, size)?;
        written += write_util::write_u16(output, self.flags)?;
        written += write_util::write_u32(output, self.name_index)?;
        written += self.value.write(output)?;
        Ok(written)
    }
}

impl ArscSerializable for Header {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut written = write_util::write_u16(output, self.resource_type as u16)?;
        written += write_util::write_u16(output, self.header_size)?;
        written += write_util::write_u32(output, self.size)?;
        Ok(written)
    }
}

impl ArscSerializable for StringPool {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = self.header().write(output)?;
        position += write_util::write_u32(output, self.strings.len())?;
        position += write_util::write_u32(output, self.styles.len())?; // style_count
        position += write_util::write_u32(output, self.flags)?;
        let string_start = 7 * 4 + self.strings.len() * 4 + self.styles.len() * 4;
        position += write_util::write_u32(output, string_start)?; // string_offset

        let strings = self.strings.iter().map(String::as_ref);
        let string_length = if self.use_utf8() {
            strings.map(StringPool::utf8_string_size).sum()
        } else {
            strings.map(StringPool::utf16_string_size).sum()
        };
        let string_padding = padding(string_length);
        let style_start = if self.styles.is_empty() {
            0
        } else {
            string_start + string_length + string_padding
        };
        position += write_util::write_u32(output, style_start)?;
        position += self.write_string_offsets(output)?;
        position += self.write_style_offsets(output)?;
        position += self.write_strings(output)?;
        position += output.write(&vec![0; string_padding])?;
        for style in &self.styles {
            position += style.write(output)?;
        }
        if !self.styles.is_empty() {
            position += write_util::write_u32(output, u32::MAX)?;
            position += write_util::write_u32(output, u32::MAX)?;
        }

        Ok(position)
    }
}

impl StringPool {
    fn write_string_offsets<W: Write>(&self, output: &mut W) -> Result<usize> {
        if self.use_utf8() {
            self.write_utf8_offsets(output)
        } else {
            self.write_utf16_offsets(output)
        }
    }

    fn write_style_offsets<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = 0;
        let mut written = 0;
        for style in &self.styles {
            written += write_util::write_u32(output, position)?;
            position += style.spans.len() * StyleSpan::SIZE + 4;
        }
        Ok(written)
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
            if self.use_utf8() {
                let char_count = string.chars().count();
                position += Self::write_utf8_length(buffer, char_count)?;
                position += Self::write_utf8_length(buffer, string.len())?;
                position += buffer.write(string.as_bytes())?;
                position += write_util::write_u8(buffer, 0)?;
            } else {
                position += Self::write_utf16_length(buffer, string.encode_utf16().count())?;
                position += write_util::write_string_utf16(buffer, string)?;
                position += write_util::write_u16(buffer, 0)?;
            }
        }
        Ok(position)
    }

    fn write_utf8_length<W: Write>(buffer: &mut W, length: usize) -> Result<usize> {
        let mut offset = 0;
        if length > 0x7F {
            offset += write_util::write_u8(buffer, (length >> 8) | 0x80)?;
        }
        offset += write_util::write_u8(buffer, length & 0xFF)?;
        Ok(offset)
    }

    fn write_utf16_length<W: Write>(buffer: &mut W, length: usize) -> Result<usize> {
        let mut offset = 0;
        if length > 0x7FFF {
            let leading_two_bytes = (length >> 16) | 0x8000;
            offset += write_util::write_u8(buffer, leading_two_bytes & 0xFF)?;
            offset += write_util::write_u8(buffer, leading_two_bytes >> 8)?;
        }
        offset += write_util::write_u8(buffer, length & 0xFF)?;
        offset += write_util::write_u8(buffer, (length >> 8) & 0xFF)?;
        Ok(offset)
    }
}

impl ArscSerializable for Style {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut written = 0;
        for span in &self.spans {
            written += span.write(output)?;
        }
        written += write_util::write_u32(output, u32::MAX)?;
        Ok(written)
    }
}

impl ArscSerializable for StyleSpan {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = write_util::write_u32(output, self.name)?;
        position += write_util::write_u32(output, self.start)?;
        position += write_util::write_u32(output, self.end)?;
        Ok(position)
    }
}

impl ArscSerializable for Specs {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = self.header().write(output)?;
        position += write_util::write_u8(output, self.type_id)?;
        position += write_util::write_u8(output, self.res0)?;
        position += write_util::write_u16(output, self.res1)?;
        position += write_util::write_u32(output, self.specs.len())?;
        for spec in &self.specs {
            position += spec.write(output)?;
        }
        Ok(position)
    }
}

impl ArscSerializable for Spec {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        write_util::write_u32(output, self.flags)
    }
}

impl ArscSerializable for Config {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = self.header().write(output)?;
        position += write_util::write_u8(output, self.type_id)?;
        position += write_util::write_u8(output, self.res0)?;
        position += write_util::write_u16(output, self.res1)?;
        position += write_util::write_u32(output, self.resources.entry_count())?;
        let entry_start = position
            + 4
            + self.id.len()
            + padding(self.id.len())
            + self.resources.entry_count() * 4;
        position += write_util::write_u32(output, entry_start)?;
        position += output.write(&self.id)?;
        position += output.write(&vec![0; padding(self.id.len())])?;
        position += self.resources.write(output)?;

        Ok(position)
    }
}

impl ArscSerializable for Resources {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = self.write_entries(output)?;
        for resource in &self.resources {
            position += resource.write(output)?;
        }
        Ok(position)
    }
}

impl Resources {
    fn write_entries<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut offset = 0;
        let mut written = 0;

        let mut expected_spec_id = 0;
        for resource in &self.resources {
            let gaps = resource.spec_id - expected_spec_id;
            for _ in 0..gaps {
                written += write_util::write_u32(output, u32::MAX)?;
            }
            written += write_util::write_u32(output, offset)?;
            offset += resource.size();
            expected_spec_id = resource.spec_id + 1;
        }
        for _ in expected_spec_id..self.entry_count() {
            written += write_util::write_u32(output, u32::MAX)?;
        }
        Ok(written)
    }
}

impl ArscSerializable for Type {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = 0;
        for specs in &self.specs {
            position += specs.write(output)?;
        }
        for config in &self.configs {
            position += config.write(output)?;
        }
        Ok(position)
    }
}

impl ArscSerializable for Package {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = self.header().write(output)?;
        position += write_util::write_u32(output, self.id)?;
        let string_written = write_util::write_string_utf16(output, &self.name)?;
        position += string_written;
        if string_written < 256 {
            position += output.write(&vec![0; 256 - string_written])?;
        }

        let type_string_offset = position + 5 * 4;
        position += write_util::write_u32(output, type_string_offset)?; // type_string_offset
        position += write_util::write_u32(output, self.last_public_type)?; // last_public_type
        let key_string_offset = type_string_offset + self.type_names.size();
        position += write_util::write_u32(output, key_string_offset)?; // key_string_offset
        position += write_util::write_u32(output, self.last_public_key)?; // last_public_key
        position += write_util::write_u32(output, 0)?; // type_id_offset

        position += self.type_names.write(output)?;
        position += self.key_names.write(output)?;
        for r#type in &self.types {
            position += r#type.write(output)?;
        }
        Ok(position)
    }
}

impl ArscSerializable for Arsc {
    fn write<W: Write>(&self, output: &mut W) -> Result<usize> {
        let mut position = self.header().write(output)?;
        position += write_util::write_u32(output, self.packages.len())?;
        position += self.global_string_pool.write(output)?;
        for package in &self.packages {
            position += package.write(output)?;
        }
        Ok(position)
    }
}
