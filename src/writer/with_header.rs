use super::components_sizing::ByteSizing;
use crate::components::{Arsc, Config, Header, Package, Specs, StringPool, TypeFlag};

pub(in crate::writer) trait WithHeader: ByteSizing {
    const HEADER_SIZE: u16;
    const TYPE_FLAG: TypeFlag;

    fn header(&self) -> Header {
        Header {
            type_flag: Self::TYPE_FLAG,
            header_size: Self::HEADER_SIZE,
            size: self.size() as u64,
        }
    }
}

impl WithHeader for Arsc {
    const HEADER_SIZE: u16 = 0x000C;
    const TYPE_FLAG: TypeFlag = TypeFlag::ResTableType;
}

impl WithHeader for StringPool {
    const HEADER_SIZE: u16 = 0x001C;
    const TYPE_FLAG: TypeFlag = TypeFlag::ResStringPoolType;
}

impl WithHeader for Package {
    const HEADER_SIZE: u16 = 0x011C;
    const TYPE_FLAG: TypeFlag = TypeFlag::ResTablePackageType;
}

impl WithHeader for Specs {
    const HEADER_SIZE: u16 = 0x0010;
    const TYPE_FLAG: TypeFlag = TypeFlag::ResTableTypeSpecType;
}

impl WithHeader for Config {
    const HEADER_SIZE: u16 = 0x0040;
    const TYPE_FLAG: TypeFlag = TypeFlag::ResTableTypeType;
}
