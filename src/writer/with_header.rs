use super::components_sizing::ByteSizing;
use crate::components::{Arsc, Config, Header, Package, ResourceType, Specs, StringPool};

pub(in crate::writer) trait WithHeader: ByteSizing {
    const HEADER_SIZE: u16;
    const TYPE_FLAG: ResourceType;

    fn header(&self) -> Header {
        Header {
            resource_type: Self::TYPE_FLAG,
            header_size: Self::HEADER_SIZE,
            size: self.size() as u64,
        }
    }
}

impl WithHeader for Arsc {
    const HEADER_SIZE: u16 = 0x000C;
    const TYPE_FLAG: ResourceType = ResourceType::Table;
}

impl WithHeader for StringPool {
    const HEADER_SIZE: u16 = 0x001C;
    const TYPE_FLAG: ResourceType = ResourceType::StringPool;
}

impl WithHeader for Package {
    const HEADER_SIZE: u16 = 0x0120;
    const TYPE_FLAG: ResourceType = ResourceType::TablePackage;
}

impl WithHeader for Specs {
    const HEADER_SIZE: u16 = 0x0010;
    const TYPE_FLAG: ResourceType = ResourceType::TableTypeSpec;
}

impl WithHeader for Config {
    const HEADER_SIZE: u16 = 0x0054;
    const TYPE_FLAG: ResourceType = ResourceType::TableType;
}
