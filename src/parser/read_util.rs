use paste::paste;
use std::io::{Read, Result, Seek, SeekFrom};

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

/// read 0-terminated string as utf16 encoding
/// ## Warning:
/// This function always reads `SIZE * 2` bytes
pub fn read_string_utf16<const SIZE: usize, R: Read + Seek>(reader: &mut R) -> Result<String> {
    let end = reader.stream_position()? + SIZE as u64 * 2;
    let bytes = std::iter::repeat_with(|| read_u16(reader))
        .take(SIZE)
        .take_while(|byte| byte.as_ref().ok() != Some(&0))
        .collect::<Result<Vec<_>>>()?;
    let string = String::from_utf16(&bytes).expect("Not Uft-16");
    reader.seek(SeekFrom::Start(end))?;
    Ok(string)
}
