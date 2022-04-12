use super::components_sizing::ByteSizing;
use crate::components::{Arsc, Config, Header, Package, ResourceType, Specs, StringPool};

/// A trait for objects that are chunks (with header).
/// It handles the header generation with predefined information
pub(in crate::writer) trait WithHeader: ByteSizing {
    /// The `header_size` attribute in generated header object
    fn get_header_size(&self) -> u16;
    /// The `resource_type` attribute in generated header object
    const RESOURCE_TYPE: ResourceType;

    /// Generate a header
    fn header(&self) -> Header {
        Header {
            resource_type: Self::RESOURCE_TYPE,
            header_size: self.get_header_size(),
            size: self.size() as u64,
        }
    }
}

impl WithHeader for Arsc {
    fn get_header_size(&self) -> u16 {
        0x000C
    }
    const RESOURCE_TYPE: ResourceType = ResourceType::Table;
}

impl WithHeader for StringPool {
    fn get_header_size(&self) -> u16 {
        0x001C
    }
    const RESOURCE_TYPE: ResourceType = ResourceType::StringPool;
}

impl WithHeader for Package {
    fn get_header_size(&self) -> u16 {
        0x0120
    }
    const RESOURCE_TYPE: ResourceType = ResourceType::TablePackage;
}

impl WithHeader for Specs {
    fn get_header_size(&self) -> u16 {
        self.header_size
    }
    const RESOURCE_TYPE: ResourceType = ResourceType::TableTypeSpec;
}

impl WithHeader for Config {
    fn get_header_size(&self) -> u16 {
        self.header_size
    }

    const RESOURCE_TYPE: ResourceType = ResourceType::TableType;
}
