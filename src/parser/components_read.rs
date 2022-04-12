use super::read_util;
use crate::components::{Header, ResourceType, StringPool, Value};
use crate::{
    Arsc, Config, Package, ResourceEntry, ResourceValue, Resources, Spec, Specs, Style, StyleSpan,
    Type,
};
use std::io::{BufReader, Error, Read, Seek, SeekFrom};

impl<R: Read> TryFrom<&mut BufReader<R>> for Header {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let type_bits = read_util::read_u16(reader)?;
        let r#type = ResourceType::from(type_bits);
        let header_size = read_util::read_u16(reader)?;
        let size = read_util::read_u32(reader)? as u64;
        Ok(Header {
            resource_type: r#type,
            header_size,
            size,
        })
    }
}

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for StringPool {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let base = reader.stream_position()?;
        let header = crate::Header::try_from(&mut *reader)?;
        assert_eq!(header.resource_type, crate::ResourceType::StringPool);

        let string_count = read_util::read_u32(reader)? as usize;
        let style_count = read_util::read_u32(reader)? as usize;
        let flags = read_util::read_u32(reader)?;
        let string_offset = read_util::read_u32(reader)? as u64;
        let style_offset = read_util::read_u32(reader)? as u64;
        let mut offsets = Vec::with_capacity(string_count);
        for _ in 0..string_count {
            offsets.push(read_util::read_u32(reader)? as u64)
        }
        let mut style_offsets = Vec::with_capacity(style_count);
        for _ in 0..style_count {
            style_offsets.push(read_util::read_u32(reader)? as u64)
        }
        debug_assert_eq!(reader.stream_position()?, string_offset + base);
        let mut strings = Vec::with_capacity(string_count);
        for _ in 0..string_count {
            let string = if flags & StringPool::UTF8_FLAG != 0 {
                StringPool::read_utf8_string_item(reader)?
            } else {
                StringPool::read_utf16_string_item(reader)?
            };
            strings.push(string);
        }
        reader.seek(SeekFrom::Start(base + style_offset))?;
        let styles = std::iter::repeat_with(|| Style::try_from(&mut *reader))
            .take(style_count)
            .collect::<std::io::Result<Vec<_>>>()?;
        reader.seek(SeekFrom::Start(base + header.size))?;
        Ok(StringPool {
            flags,
            strings,
            styles,
        })
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
        reader.seek(SeekFrom::Current(2))?; // skip null terminator
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
impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for Style {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let mut spans = Vec::new();
        loop {
            let name = read_util::read_u32(reader)?;
            if name == Style::RES_STRING_POOL_SPAN_END {
                break;
            }
            let start = read_util::read_u32(reader)?;
            let end = read_util::read_u32(reader)?;
            spans.push(StyleSpan { name, start, end })
        }
        Ok(Style { spans })
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

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for Specs {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let type_id = read_util::read_u8(reader)? as usize;
        let res0 = read_util::read_u8(reader)?;
        let res1 = read_util::read_u16(reader)?;
        let entry_count = read_util::read_u32(reader)? as usize;

        let specs = std::iter::repeat_with(|| read_util::read_u32(reader))
            .take(entry_count)
            .enumerate()
            .map(|(id, flags)| Result::Ok(Spec::new(flags?, id)))
            .collect::<Result<Vec<_>, Self::Error>>()?;
        debug_assert!(!specs.is_empty(), "Specs cannot be empty");
        Ok(Specs {
            type_id,
            res0,
            res1,
            specs,
            header_size: u16::MAX,
        })
    }
}

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for ResourceEntry {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let _size = read_util::read_u16(reader)?;
        let flags = read_util::read_u16(reader)?;
        let name_index = read_util::read_u32(reader)? as usize;

        let value = if flags & ResourceEntry::ENTRY_FLAG_COMPLEX != 0 {
            let parent = read_util::read_u32(reader)?;
            let count = read_util::read_u32(reader)? as usize;
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                let index = read_util::read_u32(reader)?;
                let value = Value::try_from(&mut *reader)?;
                values.push((index, value));
            }
            ResourceValue::Bag { parent, values }
        } else {
            ResourceValue::Plain(Value::try_from(reader)?)
        };
        Ok(ResourceEntry {
            flags,
            name_index,
            value,
            spec_id: usize::MAX,
        })
    }
}

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for Config {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let type_id = read_util::read_u8(reader)? as usize;
        let res0 = read_util::read_u8(reader)?;
        let res1 = read_util::read_u16(reader)?;
        let entry_count = read_util::read_u32(reader)? as usize;
        let _entry_start = read_util::read_u32(reader)?;
        let config_id = Config::parse_config_id(reader)?;

        let resources = Config::parse_config_resources(reader, entry_count)?;
        Ok(Config {
            type_id,
            res0,
            res1,
            id: config_id,
            resources,
            header_size: u16::MAX,
        })
    }
}

impl Config {
    fn parse_config_id<R: Read + Seek>(reader: &mut R) -> std::io::Result<Vec<u8>> {
        let size = read_util::read_u32(reader)? as usize;
        let mut config_id = vec![0_u8; size];
        reader.seek(SeekFrom::Current(-4))?;
        reader.read_exact(&mut config_id)?;
        Ok(config_id)
    }

    fn parse_config_resources<R: Read + Seek>(
        reader: &mut BufReader<R>,
        entry_count: usize,
    ) -> std::io::Result<Resources> {
        let _current = reader.stream_position()?;
        let entries = std::iter::repeat_with(|| read_util::read_u32(reader))
            .take(entry_count)
            .collect::<std::io::Result<Vec<_>>>()?;
        let _current = reader.stream_position()?;
        let mut resources = Vec::with_capacity(entry_count);
        for (spec_index, entry) in entries.into_iter().enumerate() {
            if entry == u32::MAX {
                continue;
            }
            let mut resource = ResourceEntry::try_from(&mut *reader)?;
            resource.spec_id = spec_index;
            resources.push(resource);
        }
        resources.shrink_to_fit();
        let resource_count = resources.len();
        Ok(Resources {
            resources,
            missing_entries: entry_count - resource_count,
        })
    }
}

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for Package {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let package_header = Header::try_from(&mut *reader)?;
        debug_assert_eq!(package_header.resource_type, ResourceType::TablePackage);
        let package_id = read_util::read_u32(reader)?;
        let package_name = Self::parse_package_name(reader)?;

        let _type_string_offset = read_util::read_u32(reader)?;
        let last_public_type = read_util::read_u32(reader)?;
        let _key_string_offset = read_util::read_u32(reader)?;
        let last_public_key = read_util::read_u32(reader)?;
        let _type_id_offset = read_util::read_u32(reader)?;

        let type_names = StringPool::try_from(&mut *reader)?;
        let mut types = (1..=type_names.strings.len())
            .map(Type::with_id)
            .collect::<Vec<_>>();
        let key_names = StringPool::try_from(&mut *reader)?;

        while let Ok(header) = Header::try_from(&mut *reader) {
            match header.resource_type {
                ResourceType::TableTypeSpec => {
                    let mut specs = Specs::try_from(&mut *reader)?;
                    specs.header_size = header.header_size;
                    debug_assert!(
                        &types[specs.type_id - 1].specs.is_none(),
                        "Target type already has specs"
                    );
                    types[specs.type_id - 1].specs.replace(specs);
                }
                ResourceType::TableType => {
                    let mut config = Config::try_from(&mut *reader)?;
                    config.header_size = header.header_size;
                    types[config.type_id - 1].configs.push(config);
                }
                flag => unreachable!("Unexpected flag: {flag:?}"),
            }
        }
        Ok(Package {
            id: package_id,
            name: package_name,
            type_names,
            last_public_type,
            types,
            key_names,
            last_public_key,
        })
    }
}

impl Package {
    fn parse_package_name<R: Read + Seek>(reader: &mut R) -> std::io::Result<String> {
        read_util::read_string_utf16::<128, R>(reader)
    }
}

impl<R: Read + Seek> TryFrom<&mut BufReader<R>> for Arsc {
    type Error = std::io::Error;

    fn try_from(reader: &mut BufReader<R>) -> Result<Self, Self::Error> {
        let _header = Header::try_from(&mut *reader)?;
        let package_count = read_util::read_u32(reader)? as usize;
        let global_string_pool = StringPool::try_from(&mut *reader)?;
        let packages = std::iter::repeat_with(|| Package::try_from(&mut *reader))
            .take(package_count)
            .collect::<Result<Vec<_>, Self::Error>>()?;
        Ok(Arsc {
            global_string_pool,
            packages,
        })
    }
}
