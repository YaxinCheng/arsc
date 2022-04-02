use super::read_util;
use crate::components::{Header, StringPool, TypeFlag, Value};
use std::io::{BufReader, Error, Read, Seek, SeekFrom};

impl<R: Read> TryFrom<&mut BufReader<R>> for Header {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let type_bits = read_util::read_u16(reader)?;
        let r#type = TypeFlag::from(type_bits);
        let header_size = read_util::read_u16(reader)?;
        let size = read_util::read_u32(reader)? as u64;
        Ok(Header {
            type_flag: r#type,
            header_size,
            size,
        })
    }
}

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for StringPool {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let base = reader.stream_position()? - 8;
        let string_count = read_util::read_u32(reader)? as usize;
        let _style_count = read_util::read_u32(reader)?;
        let flags = read_util::read_u32(reader)?;
        let string_offset = read_util::read_u32(reader)? as u64;
        let _style_offset = read_util::read_u32(reader)?;
        let mut offsets = Vec::with_capacity(string_count);
        for _ in 0..string_count {
            offsets.push(read_util::read_u32(reader)? as u64)
        }
        let base = base + string_offset;
        let mut strings = Vec::with_capacity(string_count);
        for offset in offsets {
            reader.seek(SeekFrom::Start(base + offset))?;
            let string = if flags & StringPool::UTF8_FLAG != 0 {
                StringPool::read_utf8_string_item(reader)?
            } else {
                StringPool::read_utf16_string_item(reader)?
            };
            strings.push(string);
        }
        Ok(StringPool { strings, flags })
    }
}

impl StringPool {
    fn read_utf8_string_item<R: Read + Seek>(reader: &mut BufReader<R>) -> Result<String, Error> {
        let _char_count = Self::utf8_length(reader)?; // string length
        let byte_count = Self::utf8_length(reader)?;
        let start = reader.stream_position()?;
        let mut string_bytes = Vec::with_capacity(byte_count);
        for _ in 0..byte_count {
            let byte = read_util::read_u8(reader)?;
            if byte == 0 {
                break;
            }
            string_bytes.push(byte);
        }
        reader.seek(SeekFrom::Start(start + byte_count as u64 + 1))?;
        Ok(String::from_utf8(string_bytes).expect("Not uft-8"))
    }

    fn utf8_length<R: Read>(reader: &mut BufReader<R>) -> Result<usize, Error> {
        let mut length = read_util::read_u8(reader)? as usize;
        if (length & 0x80) != 0 {
            length = ((length & 0x7F) << 8) | read_util::read_u8(reader)? as usize;
        }
        Ok(length)
    }

    fn read_utf16_string_item<R: Read + Seek>(reader: &mut BufReader<R>) -> Result<String, Error> {
        let char_count = Self::utf16_length(reader)?;
        let mut string_bytes = Vec::with_capacity(char_count);
        for _ in 0..char_count {
            string_bytes.push(read_util::read_u16(reader)?);
        }
        Ok(String::from_utf16(&string_bytes).expect("Not utf-16"))
    }

    fn utf16_length<R: Read>(reader: &mut BufReader<R>) -> Result<usize, Error> {
        let mut length = read_util::read_u16(reader)? as usize;
        if length > 0x7FFF {
            length = ((length & 0x7FFF) << 8) | read_util::read_u16(reader)? as usize;
        }
        Ok(length)
    }
}

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for Value {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let size = read_util::read_u16(reader)?;
        let zero = read_util::read_u8(reader)?;
        let r#type = read_util::read_u8(reader)?;
        let data_index = read_util::read_u32(reader)? as usize;
        Ok(Value {
            size,
            zero,
            r#type,
            data_index,
        })
    }
}
