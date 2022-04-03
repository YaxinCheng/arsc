use super::components_sizing::ByteSizing;
use crate::components::{Arsc, Header, Package, StringPool, TypeFlag};

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
    const TYPE_FLAG: TypeFlag = TypeFlag::RES_TABLE_TYPE;
}

impl WithHeader for StringPool {
    const HEADER_SIZE: u16 = 0x001C;
    const TYPE_FLAG: TypeFlag = TypeFlag::RES_STRING_POOL_TYPE;
}

impl WithHeader for Package {
    const HEADER_SIZE: u16 = 0x011C;
    const TYPE_FLAG: TypeFlag = TypeFlag::RES_TABLE_PACKAGE_TYPE;
}
