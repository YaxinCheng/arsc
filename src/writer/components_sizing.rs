use crate::components::{
    Arsc, Config, Header, Package, ResourceEntry, ResourceValue, Spec, Specs, StringPool, Type,
    Value,
};
use crate::{Style, StyleSpan};

/// A trait for objects that have constant sizes
/// when being written out in arsc format
pub(in crate::writer) trait ConstByteSizing {
    /// The number of bytes required to write out this type
    const SIZE: usize;
}

/// A trait for objects whose content affects its size
/// when being written out in arsc format
pub(in crate::writer) trait ByteSizing {
    /// The number of bytes required to write out this type
    fn size(&self) -> usize;
}

impl<T: ConstByteSizing> ByteSizing for T {
    fn size(&self) -> usize {
        T::SIZE
    }
}

impl ConstByteSizing for Value {
    const SIZE: usize = 2 + 1 + 1 + 4; // size + zero + type + data_index
}

impl ByteSizing for ResourceValue {
    fn size(&self) -> usize {
        match self {
            ResourceValue::Plain(_) => Value::SIZE,
            ResourceValue::Bag { parent: _, values } => {
                4 + 4// parent + count
                    + values.len() * (4 + Value::SIZE)
            }
        }
    }
}

impl ByteSizing for ResourceEntry {
    fn size(&self) -> usize {
        2 + 4 + self.value.size() // flags + name_index + value. `spec_id` is not read in
    }
}

impl ByteSizing for Config {
    fn size(&self) -> usize {
        Header::SIZE + 1 + 1 + 2 + 4 + 4 // type_id + res0 + res1 + entry_count + _entry_start
            + self.id.len()// config_id
            + padding(self.id.len())// config_id_padding
            + self.entry_count * 4 // entries
            + self.resources.len() * 2// _size + name_index 
            + self
                .resources
                .values()
                .map(ByteSizing::size)
                .sum::<usize>()
    }
}

impl ConstByteSizing for Spec {
    const SIZE: usize = 4; // flags. name_index is handled in config part
}

impl ByteSizing for Specs {
    fn size(&self) -> usize {
        // parse_spec: header + type_id + _res0 + _res1 + entry_count + sizeOf(specs)
        Header::SIZE + 1 + 1 + 2 + 4 + self.specs.len() * Spec::SIZE
    }
}

impl ByteSizing for Type {
    fn size(&self) -> usize {
        let parse_spec = self.specs.as_ref().map(ByteSizing::size).unwrap_or(0);
        let parse_config = self.configs.iter().map(ByteSizing::size).sum::<usize>();
        parse_spec + parse_config
    }
}

impl ByteSizing for StringPool {
    fn size(&self) -> usize {
        let size = Header::SIZE // header
        + 5 * 4 //string_count, style_count, flags, string_offset, style_offset
        + self.strings.len() * 4 // offsets array
        + self.styles.len() * 4; // style offsets array
        let string_length = self
            .strings
            .iter()
            .map(|s| {
                if self.use_utf8() {
                    StringPool::utf8_string_size(s)
                } else {
                    StringPool::utf16_string_size(s)
                }
            })
            .sum::<usize>();
        let string_padding = padding(string_length);
        let style_size = if self.styles.is_empty() {
            0
        } else {
            self.styles.iter().map(ByteSizing::size).sum::<usize>() + 8 // 2 extra terminals
        };
        size + string_length + string_padding + style_size
    }
}

impl StringPool {
    pub(crate) fn utf8_string_size(string: &str) -> usize {
        let char_count = string.chars().count();
        let char_count_bytes = if char_count <= 0x7F { 1 } else { 2 };

        let byte_count = string.len();
        let byte_count_bytes = if byte_count <= 0x7F { 1 } else { 2 };

        char_count_bytes + byte_count_bytes + byte_count + 1 // 1 is the null terminator
    }

    pub(crate) fn utf16_string_size(string: &str) -> usize {
        let char_count = string.chars().count();
        let char_count_bytes = if char_count <= 0x7FFF { 2 } else { 4 };

        char_count_bytes + char_count * 2 + 2 // 2 is the null terminator
    }
}

impl ByteSizing for Style {
    fn size(&self) -> usize {
        self.spans.len() * StyleSpan::SIZE + 4
    }
}

impl ConstByteSizing for StyleSpan {
    const SIZE: usize = 4 + 4 + 4;
}

impl ByteSizing for Package {
    fn size(&self) -> usize {
        Header::SIZE + 4 + 256 // header + id + package_name 
        + 5 * 4 // _type_string_offset, _last_public_type, _key_string_offset, _last_public_key, _type_id_offset
        + self.type_names.size()
        + self.types.iter().map(ByteSizing::size).sum::<usize>()
        + self.key_names.size()
    }
}

impl ByteSizing for Arsc {
    fn size(&self) -> usize {
        Header::SIZE + 4 // header + package_count
        + self.global_string_pool.size()
        + self.packages.iter().map(ByteSizing::size).sum::<usize>()
    }
}

impl ConstByteSizing for Header {
    const SIZE: usize = 8;
}

/// Calculate the padding size for a given size
///
/// # Argument:
/// * size - the size needs to be padded
/// # Returns:
/// the padding size with respect to 4
pub(crate) fn padding(size: usize) -> usize {
    (4 - size % 4) % 4
}
