use std::{
    fs::File,
    io::BufReader,
    io::{self, prelude::*},
    mem::size_of,
    str::{self, Utf8Error},
};

pub const DWORD_SIZE: usize = 4;

// turns &[u8] into a u32, f64, etc.
pub fn cast_bytes<T: ToBytes<SIZE>, const SIZE: usize>(bytes: &[u8]) -> T {
    let source: [u8; SIZE] = bytes[..SIZE].try_into().unwrap();
    T::from_le_bytes(source)
}

// this is horrible, SIZE can't be deduced in current version of rust and you'd have to manually specify it every call
// fn sized_bytes_to_bytes<T, const SIZE: usize, E: Debug>(bytes: &[u8]) -> &[u8] where T: ToBytes<SIZE> + TryInto<usize, Error = E> {
//     let size: T = cast_bytes(&bytes);
//     let size = size.try_into().unwrap();
//     &bytes[SIZE..SIZE + size]
// }

// reads the size prefix of $type and returns that many following bytes
macro_rules! sized_bytes_to_bytes {
    ($bytes:expr, $t:ident) => {{
        let size_size = std::mem::size_of::<$t>();
        let size: $t = cast_bytes(&$bytes[..size_size]);
        let size = size as usize;
        &$bytes[size_size..size_size + size]
    }};
}

// turns &[u8] with a size prefix into a string
macro_rules! bytes_to_exo_string {
    ($bytes:expr, $type:ident) => {{
        let source = sized_bytes_to_bytes!($bytes, $type);
        bytes_to_string(source)
    }};
}

pub(crate) use bytes_to_exo_string;
pub(crate) use sized_bytes_to_bytes;

pub fn read_bytes(reader: &mut BufReader<File>, size: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0; size];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn read_chunks(
    reader: &mut BufReader<File>,
    size: usize,
    chunk_size: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let buf = read_bytes(reader, size * chunk_size)?;
    let result = buf.chunks_exact(chunk_size).map(|c| c.into()).collect();

    Ok(result)
}

pub fn read_dwords(reader: &mut BufReader<File>, size: usize) -> io::Result<Vec<u32>> {
    Ok(read_chunks(reader, size, DWORD_SIZE)?
        .into_iter()
        .map(|c| cast_bytes(&c))
        .collect())
}

pub fn bytes_to_string(value: &[u8]) -> Result<String, Utf8Error> {
    str::from_utf8(value).map(|str| str.to_owned())
}

pub fn num_to_word<T: ToBytes<SIZE>, const SIZE: usize >(num: T) -> u32 {
    let mut buf = [0u8; 4];
    for (idx, byte) in num.to_le_bytes().into_iter().enumerate() {
        buf[idx] = byte;
    }
    u32::from_ne_bytes(buf)
}

pub fn bytes_to_sized_bytes<const SIZE_SIZE: usize>(bytes: &[u8]) -> Vec<u8> {
    let len = bytes.len();
    let mut buf = Vec::with_capacity(len + SIZE_SIZE);
    let len_bytes = len.to_le_bytes();
    buf.extend(&len_bytes[..SIZE_SIZE]);
    buf.extend(bytes);

    buf
}

pub trait ToUSizeVec {
    fn to_usize_vec(self) -> Vec<usize>;
}
impl ToUSizeVec for Vec<u32> {
    fn to_usize_vec(self) -> Vec<usize> {
        self.into_iter().map(|i| i as usize).collect()
    }
}

impl ToUSizeVec for &[u32] {
    fn to_usize_vec(self) -> Vec<usize> {
        self.into_iter().map(|i| *i as usize).collect()
    }
}

pub trait ToBytes<const SIZE: usize> {
    fn to_le_bytes(&self) -> [u8; SIZE];
    fn from_le_bytes(byte: [u8; SIZE]) -> Self;
}
macro_rules! impl_to_bytes {
    ($($t:ident)+) => ($(
        impl ToBytes<{ size_of::<$t>() }> for $t {
            fn to_le_bytes(&self) -> [u8; size_of::<$t>()] {
                $t::to_le_bytes(*self)
            }

            fn from_le_bytes(bytes: [u8; size_of::<$t>()]) -> $t  {
                $t::from_le_bytes(bytes)
            }
        }
    )+)
}

impl_to_bytes!(u8 i8 u16 i16 u32 i32 u64 i64 usize isize f32 f64);

