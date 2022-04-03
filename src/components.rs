use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Header {
    pub resource_type: ResourceType,
    pub header_size: u16,
    pub size: u64,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ResourceType {
    Null = 0x0000,
    StringPool = 0x0001,
    Table = 0x0002,
    TablePackage = 0x0200,
    TableType = 0x0201,
    TableTypeSpec = 0x0202,
    TableLibrary = 0x0203,
}

impl From<u16> for ResourceType {
    fn from(bits: u16) -> Self {
        use ResourceType::*;
        match bits {
            0 => Null,
            1 => StringPool,
            2 => Table,
            0x0200 => TablePackage,
            0x0201 => TableType,
            0x0202 => TableTypeSpec,
            0x0203 => TableLibrary,
            bits => unreachable!("Unexpected bits: {bits}"),
        }
    }
}

pub struct Arsc {
    pub packages: Vec<Package>,
    pub global_string_pool: StringPool,
}

pub struct Package {
    pub id: u32,
    pub name: String,
    pub type_names: StringPool,
    pub types: Vec<Type>,
    pub key_names: StringPool,
}

pub struct StringPool {
    pub strings: Vec<String>,
    pub flags: u32,
}

impl StringPool {
    pub(crate) const UTF8_FLAG: u32 = 0x00000100;
}

#[derive(Default)]
pub struct Type {
    pub id: usize, // id - 1 is the index to type_names
    pub specs: Option<Specs>,
    pub configs: Vec<Config>,
}

impl Type {
    pub fn with_id(id: usize) -> Self {
        Type {
            id,
            ..Default::default()
        }
    }
}

pub struct Specs {
    pub type_id: usize,
    pub res0: u8,
    pub res1: u16,
    pub specs: Vec<Spec>,
}

impl Specs {
    pub fn set_name_index(&mut self, spec_index: usize, name_index: usize) {
        self.specs[spec_index].name_index = name_index;
    }
}

#[derive(Default)]
pub struct Spec {
    pub flags: u32,
    pub id: usize,
    pub name_index: usize, // index to key_names
}

impl Spec {
    pub fn new(flags: u32, id: usize) -> Self {
        Spec {
            flags,
            id,
            ..Default::default()
        }
    }
}

pub struct Config {
    pub type_id: usize,
    pub res0: u8,
    pub res1: u16,
    pub entry_count: usize,
    pub id: Vec<u8>,
    pub resources: BTreeMap<usize, ResourceEntry>,
}

pub struct ResourceEntry {
    pub flags: u16,
    pub spec_id: usize, // index to spec
    pub name_index: usize,
    pub value: ResourceValue,
}

impl ResourceEntry {
    pub(crate) const ENTRY_FLAG_COMPLEX: u16 = 0x0001;
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResourceValue {
    Bag {
        parent: u32,
        values: Vec<(u32, Value)>,
    },
    Plain(Value),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Value {
    pub size: u16,
    pub zero: u8,
    pub r#type: u8,
    pub data_index: usize, // index in global_string_pool
}

impl Value {
    #[allow(dead_code)]
    pub(crate) const TYPE_STRING: u8 = 0x03;
}
