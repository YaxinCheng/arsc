use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Header {
    pub type_flag: TypeFlag,
    pub header_size: u16,
    pub size: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeFlag {
    RES_NULL_TYPE = 0x0000,
    RES_STRING_POOL_TYPE = 0x0001,
    RES_TABLE_TYPE = 0x0002,
    RES_TABLE_PACKAGE_TYPE = 0x0200,
    RES_TABLE_TYPE_TYPE = 0x0201,
    RES_TABLE_TYPE_SPEC_TYPE = 0x0202,
    RES_TABLE_LIBRARY_TYPE = 0x0203,
}

impl From<u16> for TypeFlag {
    fn from(bits: u16) -> Self {
        use TypeFlag::*;
        match bits {
            0 => RES_NULL_TYPE,
            1 => RES_STRING_POOL_TYPE,
            2 => RES_TABLE_TYPE,
            0x0200 => RES_TABLE_PACKAGE_TYPE,
            0x0201 => RES_TABLE_TYPE_TYPE,
            0x0202 => RES_TABLE_TYPE_SPEC_TYPE,
            0x0203 => RES_TABLE_LIBRARY_TYPE,
            bits => unreachable!("Unexpected bits: {bits}"),
        }
    }
}

pub struct Package {
    pub id: u32,
    pub name: String,
    pub global_string_pool: StringPool,
    pub types: Types,
    pub key_names: StringPool,
}

pub struct StringPool {
    pub strings: Vec<String>,
    pub flags: u32,
}

pub struct Types {
    pub flags: u32,
    pub types: Vec<Type>,
}

impl From<StringPool> for Types {
    fn from(string_pool: StringPool) -> Self {
        let flags = string_pool.flags;
        let types = string_pool
            .strings
            .into_iter()
            .enumerate()
            .map(|(id, name)| Type {
                id: id + 1,
                name,
                ..Default::default()
            })
            .collect();
        Self { flags, types }
    }
}

#[derive(Default)]
pub struct Type {
    pub id: usize,
    pub name: String,
    pub specs: Vec<ResSpec>,
    pub configs: Vec<Config>,
}

#[derive(Default)]
pub struct ResSpec {
    pub res0: u8,
    pub res1: u16,
    pub flags: u32,
    pub id: usize,
    pub name_index: usize,
}

pub struct Config {
    pub res0: u8,
    pub res1: u16,
    pub entry_count: usize,
    pub id: Vec<u8>,
    pub resources: BTreeMap<usize, ResourceEntry>,
}

pub struct ResourceEntry {
    pub flags: u16,
    pub spec_id: usize,
    pub value: ResourceValue,
}

pub enum ResourceValue {
    Bag {
        parent: u32,
        values: Vec<(u32, Value)>,
    },
    Plain(Value),
}

pub struct Value {
    pub size: u16,
    pub zero: u8,
    pub r#type: u8,
    pub data_index: usize,
}

impl Value {
    pub(crate) const TYPE_STRING: u8 = 0x03;
}
