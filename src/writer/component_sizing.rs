use crate::components::{
    Arsc, Config, Package, ResourceEntry, ResourceValue, Spec, StringPool, Type, Value,
};

pub(in crate::writer) trait ByteSizing {
    fn size(&self) -> usize;
}

impl ByteSizing for Value {
    fn size(&self) -> usize {
        2 + 1 + 1 + 4 // size + zero + type + data_index
    }
}

impl ByteSizing for ResourceValue {
    fn size(&self) -> usize {
        match self {
            ResourceValue::Plain(value) => value.size(),
            ResourceValue::Bag { parent: _, values } => {
                4 + 4// parent + count
                    + values.len() * values
                        .first()
                        .map(|(_, value)| 4 + value.size())
                        .unwrap_or(0)
            }
        }
    }
}

impl ByteSizing for ResourceEntry {
    fn size(&self) -> usize {
        2 + self.value.size() // flags + value. `spec_id` is not read in
    }
}

impl ByteSizing for Config {
    fn size(&self) -> usize {
        let config_id_padding = if self.id.len() % 4 == 0 {
            0
        } else {
            4 - self.id.len() % 4
        };
        1 + 1 + 2 + 4 + 4 // type_id + res0 + res1 + entry_count + _entry_start
            + self.id.len()// config_id
            + config_id_padding
            + self.entry_count * 4 // entries
            + self.resources.len() * (2 + 4)// _size + name_index 
            + self
                .resources
                .values()
                .map(ByteSizing::size)
                .sum::<usize>()
    }
}

impl ByteSizing for Spec {
    fn size(&self) -> usize {
        4 // flags. name_index is handled in config part
    }
}

impl ByteSizing for Type {
    fn size(&self) -> usize {
        // parse_spec: type_id + _res0 + _res1 + entry_count + sizeOf(specs)
        let parse_spec = if self.specs.is_empty() {
            0
        } else {
            1 + 1 + 2 + 4 + self.specs.len() * self.specs[0].size()
        };
        let parse_config = self.configs.iter().map(ByteSizing::size).sum::<usize>();
        // one header dropped for each config
        let headers_for_configs = 8 * self.configs.len();
        // one header for spec if exists
        let headers_for_spec = if self.specs.is_empty() { 0 } else { 8 };
        parse_spec + parse_config + headers_for_configs + headers_for_spec
    }
}

impl ByteSizing for StringPool {
    fn size(&self) -> usize {
        let size = 8 // header
        + 5 * 4 //string_count, _style_count, flags, string_offset, _style_offset
        + self.strings.len() * 4 // offsets array
        + self.strings.iter().map(|s| if (self.flags & StringPool::UTF8_FLAG) != 0 {
            StringPool::utf8_string_size(s)
        } else {
            StringPool::utf16_string_size(s)
        }).sum::<usize>();
        let padding = if size % 4 != 0 { 4 - size % 4 } else { 0 };
        size + padding
    }
}

impl StringPool {
    fn utf8_string_size(string: &str) -> usize {
        let char_count = string.chars().count();
        let char_count_bytes = if char_count <= 0x7F { 1 } else { 2 };

        let byte_count = string.len();
        let byte_count_bytes = if byte_count <= 0x7F { 1 } else { 2 };

        char_count_bytes + byte_count_bytes + byte_count + 1 // 1 is the null terminator
    }

    fn utf16_string_size(string: &str) -> usize {
        let char_count = string.chars().count();
        let char_count_bytes = if char_count <= 0x7FFF { 2 } else { 4 };

        char_count_bytes + char_count * 2 + 2 // 2 is the null terminator
    }
}

impl ByteSizing for Package {
    fn size(&self) -> usize {
        8 + 4 + 256 // header + id + package_name 
        + 5 * 4 // _type_string_offset, _last_public_type, _key_string_offset, _last_public_key, _type_id_offset
        + self.type_names.size()
        + self.types.iter().map(ByteSizing::size).sum::<usize>()
        + self.key_names.size()
    }
}

impl ByteSizing for Arsc {
    fn size(&self) -> usize {
        8 + 4 // header + package_count
        + self.global_string_pool.size()
        + self.packages.iter().map(ByteSizing::size).sum::<usize>()
    }
}
