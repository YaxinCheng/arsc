use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Header {
    pub type_flag: TypeFlag,
    pub header_size: u16,
    pub size: u64,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TypeFlag {
    ResNullType = 0x0000,
    ResStringPoolType = 0x0001,
    ResTableType = 0x0002,
    ResTablePackageType = 0x0200,
    ResTableTypeType = 0x0201,
    ResTableTypeSpecType = 0x0202,
    ResTableLibraryType = 0x0203,
}

impl From<u16> for TypeFlag {
    fn from(bits: u16) -> Self {
        use TypeFlag::*;
        match bits {
            0 => ResNullType,
            1 => ResStringPoolType,
            2 => ResTableType,
            0x0200 => ResTablePackageType,
            0x0201 => ResTableTypeType,
            0x0202 => ResTableTypeSpecType,
            0x0203 => ResTableLibraryType,
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
    pub data_index: usize, // index in global_string_pool
}

impl Value {
    #[allow(dead_code)]
    pub(crate) const TYPE_STRING: u8 = 0x03;
}
