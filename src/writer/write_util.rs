use paste::paste;
use std::io::{Result, Write};

macro_rules! write_num {
    ($num_type: ty) => {
        paste! {
        pub fn [<write_ $num_type>]<W: Write, I: TryInto<$num_type> + Copy>(writer: &mut W, data: I) -> Result<usize> {
            let data = data.try_into().unwrap_or_else(|_| panic!(concat!("Cannot convert to", stringify!($num_type))));
            writer.write(&data.to_le_bytes())
        }
        }
    };
}

write_num!(u8);
write_num!(u16);
write_num!(u32);

pub fn write_string_utf16<W: Write>(writer: &mut W, string: &str) -> Result<usize> {
    let mut written = 0;
    for char in string.encode_utf16() {
        written += write_u16(writer, char)?;
    }
    Ok(written)
}
