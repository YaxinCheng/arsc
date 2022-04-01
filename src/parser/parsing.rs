use super::read_util;
use crate::components::{
    Config, Package, ResSpec, ResourceEntry, ResourceValue, StringPool, Type, TypeFlag, Types,
    Value,
};
use crate::Header;
use std::collections::BTreeMap;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};

pub struct Parser<R: Read + Seek>(BufReader<R>);

const ENTRY_FLAG_COMPLEX: u16 = 0x0001;

impl<R: Read + Seek> Parser<R> {
    pub fn new(reader: R) -> Self {
        Parser(BufReader::new(reader))
    }

    pub fn parse(&mut self) -> Result<Vec<Package>> {
        let header = Header::try_from(&mut self.0)?;
        println!("{header:?}");
        let package_count = read_util::read_u32(&mut self.0)? as usize;
        let mut packages = Vec::with_capacity(package_count);
        for _ in 0..package_count {
            packages.push(self.parse_package()?)
        }
        Ok(packages)
    }

    fn parse_package(&mut self) -> Result<Package> {
        let global_string_pool = self.parse_string_pool()?;
        let package_header = Header::try_from(&mut self.0)?;
        assert_eq!(package_header.type_flag, TypeFlag::RES_TABLE_PACKAGE_TYPE);
        let package_id = read_util::read_u32(&mut self.0)?;
        let package_name = self.parse_package_name()?;

        let _type_string_offset = read_util::read_u32(&mut self.0)?;
        let _last_public_type = read_util::read_u32(&mut self.0)?;
        let _key_string_offset = read_util::read_u32(&mut self.0)?;
        let _last_public_key = read_util::read_u32(&mut self.0)?;
        let _type_id_offset = read_util::read_u32(&mut self.0)?;

        let type_names = self.parse_string_pool()?;
        let mut types = Types::from(type_names);
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
            global_string_pool,
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

    fn parse_specs(&mut self, types: &mut Types) -> Result<()> {
        let type_id = read_util::read_u8(&mut self.0)? as usize;
        let res0 = read_util::read_u8(&mut self.0)?;
        let res1 = read_util::read_u16(&mut self.0)?;
        let entry_count = read_util::read_u32(&mut self.0)? as usize;

        let target_type = &mut types.types[type_id - 1];
        for spec_id in 0..entry_count {
            let flags = read_util::read_u32(&mut self.0)?;
            target_type.specs.push(ResSpec {
                res0,
                res1,
                flags,
                id: spec_id,
                ..Default::default()
            });
        }
        Ok(())
    }

    fn parse_config(&mut self, types: &mut Types) -> Result<()> {
        let start_pos = self.0.stream_position()? - 8;
        let type_id = read_util::read_u8(&mut self.0)? as usize;
        let res0 = read_util::read_u8(&mut self.0)?;
        let res1 = read_util::read_u16(&mut self.0)?;
        let entry_count = read_util::read_u32(&mut self.0)? as usize;
        let entry_start = read_util::read_u32(&mut self.0)? as u64;
        let config_id = self.parse_config_id()?;

        let resource_type = &mut types.types[type_id - 1];
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
        let size = read_util::read_u32(&mut self.0)? as usize;
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
        let mut entries = vec![-1; entry_count];
        for index in 0..entry_count {
            entries[index] = read_util::read_i32(&mut self.0)?;
        }
        let mut resources = BTreeMap::new();
        for (spec_index, entry) in entries.into_iter().enumerate() {
            if entry <= -1 {
                continue;
            }
            self.0
                .seek(SeekFrom::Start(start_pos + entry_start + entry as u64))?;
            let _size = read_util::read_u16(&mut self.0)?;
            let flags = read_util::read_u16(&mut self.0)?;
            res_type.specs[spec_index].name_index = read_util::read_u32(&mut self.0)? as usize;
            let resource = self.parse_res_entry(flags, spec_index)?;
            resources.insert(spec_index, resource);
        }
        Ok(resources)
    }

    fn parse_res_entry(&mut self, flags: u16, spec_id: usize) -> Result<ResourceEntry> {
        let value = if flags & ENTRY_FLAG_COMPLEX != 0 {
            let parent = read_util::read_u32(&mut self.0)?;
            let count = read_util::read_u32(&mut self.0)? as usize;
            let mut values = Vec::with_capacity(count);
            for _ in 0..count {
                let index = read_util::read_u32(&mut self.0)?;
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
