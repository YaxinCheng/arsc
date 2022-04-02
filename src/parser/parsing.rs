use super::read_util;
use crate::components::{
    Arsc, Config, Header, Package, ResourceEntry, ResourceValue, Spec, StringPool, Type, TypeFlag,
    Value,
};
use std::collections::BTreeMap;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};

pub struct Parser<R: Read>(BufReader<R>);

const ENTRY_FLAG_COMPLEX: u16 = 0x0001;

impl<R: Read + Seek> Parser<R> {
    pub fn new(reader: R) -> Self {
        Parser(BufReader::new(reader))
    }

    pub fn parse(&mut self) -> Result<Arsc> {
        let _header = Header::try_from(&mut self.0)?;
        let package_count = self.read_u32()? as usize;
        let global_string_pool = self.parse_string_pool()?;
        let packages = std::iter::repeat_with(|| self.parse_package())
            .take(package_count)
            .collect::<Result<Vec<_>>>()?;
        Ok(Arsc {
            global_string_pool,
            packages,
        })
    }

    fn parse_package(&mut self) -> Result<Package> {
        let package_header = Header::try_from(&mut self.0)?;
        debug_assert_eq!(package_header.type_flag, TypeFlag::RES_TABLE_PACKAGE_TYPE);
        let package_id = self.read_u32()?;
        let package_name = self.parse_package_name()?;

        let _type_string_offset = self.read_u32()?;
        let _last_public_type = self.read_u32()?;
        let _key_string_offset = self.read_u32()?;
        let _last_public_key = self.read_u32()?;
        let _type_id_offset = self.read_u32()?;

        let type_names = self.parse_string_pool()?;
        let mut types = (1..=type_names.strings.len())
            .map(|id| Type {
                id,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let key_names = self.parse_string_pool()?;

        while let Ok(header) = Header::try_from(&mut self.0) {
            match header.type_flag {
                TypeFlag::RES_TABLE_TYPE_SPEC_TYPE => self.parse_specs(&mut types)?,
                TypeFlag::RES_TABLE_TYPE_TYPE => self.parse_config(&mut types)?,
                flag => unreachable!("Unexpected flag: {flag:?}"),
            }
        }
        Ok(Package {
            id: package_id,
            name: package_name,
            type_names,
            types,
            key_names,
        })
    }

    fn parse_string_pool(&mut self) -> Result<StringPool> {
        let start = self.0.stream_position()?;

        let header = Header::try_from(&mut self.0)?;
        assert_eq!(header.type_flag, TypeFlag::RES_STRING_POOL_TYPE);
        let string_pool = StringPool::try_from(&mut self.0)?;

        self.0.seek(SeekFrom::Start(start + header.size))?;

        Ok(string_pool)
    }

    fn parse_package_name(&mut self) -> Result<String> {
        read_util::read_string_utf16::<128, BufReader<R>>(&mut self.0)
    }

    fn parse_specs(&mut self, types: &mut [Type]) -> Result<()> {
        let type_id = self.read_u8()? as usize;
        let _res0 = self.read_u8()?;
        let _res1 = self.read_u16()?;
        let entry_count = self.read_u32()? as usize;

        let target_type = &mut types[type_id - 1];
        for spec_id in 0..entry_count {
            let flags = self.read_u32()?;
            target_type.specs.push(Spec {
                flags,
                id: spec_id,
                ..Default::default()
            });
        }
        Ok(())
    }

    fn parse_config(&mut self, types: &mut [Type]) -> Result<()> {
        let start_pos = self.0.stream_position()? - 8;
        let type_id = self.read_u8()? as usize;
        let res0 = self.read_u8()?;
        let res1 = self.read_u16()?;
        let entry_count = self.read_u32()? as usize;
        let entry_start = self.read_u32()? as u64;
        let config_id = self.parse_config_id()?;

        let resource_type = &mut types[type_id - 1];
        let resources =
            self.parse_config_resources(start_pos, entry_start, entry_count, resource_type)?;
        let config = Config {
            res0,
            res1,
            entry_count,
            id: config_id,
            resources,
        };
        resource_type.configs.push(config);
        Ok(())
    }

    fn parse_config_id(&mut self) -> Result<Vec<u8>> {
        let position = self.0.stream_position()?;
        let size = self.read_u32()? as usize;
        let mut config_id = vec![0_u8; size];
        self.0.seek(SeekFrom::Start(position))?;
        self.0.read_exact(&mut config_id)?;
        Ok(config_id)
    }

    fn parse_config_resources(
        &mut self,
        start_pos: u64,
        entry_start: u64,
        entry_count: usize,
        res_type: &mut Type,
    ) -> Result<BTreeMap<usize, ResourceEntry>> {
        let entries = std::iter::repeat_with(|| self.read_i32())
            .take(entry_count)
            .collect::<Result<Vec<_>>>()?;
        let mut resources = BTreeMap::new();
        for (spec_index, entry) in entries.into_iter().enumerate() {
            if entry <= -1 {
                continue;
            }
            self.0
                .seek(SeekFrom::Start(start_pos + entry_start + entry as u64))?;
            let _size = self.read_u16()?;
            let flags = self.read_u16()?;
            res_type.specs[spec_index].name_index = self.read_u32()? as usize;
            let resource = self.parse_res_entry(flags, spec_index)?;
            resources.insert(spec_index, resource);
        }
        Ok(resources)
    }

    fn parse_res_entry(&mut self, flags: u16, spec_id: usize) -> Result<ResourceEntry> {
        let value = if flags & ENTRY_FLAG_COMPLEX != 0 {
            let parent = self.read_u32()?;
            let count = self.read_u32()? as usize;
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                let index = self.read_u32()?;
                let value = Value::try_from(&mut self.0)?;
                values.push((index, value));
            }
            ResourceValue::Bag { parent, values }
        } else {
            ResourceValue::Plain(Value::try_from(&mut self.0)?)
        };
        Ok(ResourceEntry {
            flags,
            spec_id,
            value,
        })
    }
}

macro_rules! impl_read {
    ($type: ty) => {
        paste::paste! {
            fn [<read_ $type>](&mut self) -> Result<$type> {
                read_util::[<read_ $type>](&mut self.0)
            }
        }
    };
}

impl<R: Read> Parser<R> {
    impl_read!(u32);
    impl_read!(u16);
    impl_read!(u8);
    impl_read!(i32);
}
