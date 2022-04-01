use paste::paste;
use std::io::{Read, Result};
use std::ops::Index;

macro_rules! read_num {
    ($num_type: ty) => {
        paste! {
        pub fn [<read_ $num_type>]<R: Read>(reader: &mut R) -> Result<$num_type> {
            let mut bytes = [0_u8; std::mem::size_of::<$num_type>()];
            reader.read_exact(&mut bytes)?;
            Ok(<$num_type>::from_le_bytes(bytes))
        }
        }
    };
}

read_num!(u8);
read_num!(u16);
read_num!(u32);
read_num!(i32);

pub fn read_string_utf16<const SIZE: usize, R: Read>(reader: &mut R) -> Result<String> {
    let mut bytes = [0_u16; SIZE];
    for offset in 0..SIZE {
        bytes[offset] = read_u16(reader)?;
    }
    let index_of_zero = bytes.iter().position(|byte| byte == &0).unwrap_or(SIZE);
    Ok(String::from_utf16(&bytes[..index_of_zero]).expect("Not Uft-16"))
}
