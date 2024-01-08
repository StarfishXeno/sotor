use bytemuck::{bytes_of, cast_slice, pod_read_unaligned, AnyBitPattern, NoUninit};
use std::{
    fmt::Debug,
    io::{self, prelude::*},
    mem::size_of,
};

pub const DWORD_SIZE: usize = 4;

// turns &[u8] into a u32, f64, etc.
pub fn cast_bytes<T: AnyBitPattern>(bytes: &[u8]) -> T {
    pod_read_unaligned(&bytes[..size_of::<T>()])
}

// reads the size prefix of type S and returns that many following bytes
pub fn sized_bytes_to_bytes<S: AnyBitPattern + NoUninit + TryInto<usize, Error = impl Debug>>(
    bytes: &[u8],
) -> Vec<u8> {
    let size_size = size_of::<S>();
    let size: usize = cast_bytes::<S>(bytes).try_into().unwrap();
    bytes[size_size..size_size + size].to_vec()
}

// turns &[u8] with a size prefix into a string
pub fn sized_bytes_to_string<S: AnyBitPattern + NoUninit + TryInto<usize, Error = impl Debug>>(
    bytes: &[u8],
) -> String {
    let source = sized_bytes_to_bytes::<S>(bytes);
    bytes_to_string(&source)
}

// returns a buffer with <bytes> prefixed with it's length
pub fn bytes_to_sized_bytes<const SIZE_SIZE: usize>(bytes: &[u8]) -> Vec<u8> {
    let len = bytes.len();
    let mut buf = Vec::with_capacity(len + SIZE_SIZE);
    let len_bytes = len.to_le_bytes();
    buf.extend(&len_bytes[..SIZE_SIZE]);
    buf.extend(bytes);

    buf
}

// reads <count> bytes into a buffer
pub fn read_bytes<R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0; count];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}

// reads <count> Ts into a buffer
pub fn read_t<T: AnyBitPattern, R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<T>> {
    if count == 0 {
        return Ok(vec![]);
    };
    let bytes = read_bytes(reader, count * size_of::<T>())?.into_boxed_slice();

    Ok(cast_slice(&bytes).to_vec())
}

// reads <count> DWORDs into a buffer
pub fn read_dwords<R: Read>(reader: &mut R, count: usize) -> io::Result<Vec<u32>> {
    read_t(reader, count)
}

// reads all bytes until char is encountered, err if it's not present
pub fn read_until<R: BufRead>(reader: &mut R, char: u8) -> io::Result<Vec<u8>> {
    let mut buf = vec![];
    reader.read_until(char, &mut buf)?;
    let last_char = buf.pop();
    if last_char.is_none() || last_char.unwrap() != char {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "target char not found",
        ));
    };
    Ok(buf)
}

pub fn bytes_to_string(value: &[u8]) -> String {
    String::from_utf8_lossy(value).to_string()
}

// turns numerics (u16, f32, etc) into a u32, T can't be more than 4 bytes
pub fn num_to_dword<T: AnyBitPattern + NoUninit>(num: T) -> u32 {
    debug_assert!(size_of::<T>() <= 4, "T can't be larger than 4 bytes");
    let mut buf = [0u8; 4];
    for (idx, byte) in bytes_of(&num).iter().enumerate() {
        buf[idx] = *byte;
    }
    u32::from_ne_bytes(buf)
}

pub trait ToUsizeVec {
    fn to_usize_vec(self) -> Vec<usize>;
}
impl<T, E> ToUsizeVec for &[T]
where
    T: TryInto<usize, Error = E> + Clone,
    E: Debug,
{
    fn to_usize_vec(self) -> Vec<usize> {
        self.iter()
            .map(|i| (*i).clone().try_into().unwrap())
            .collect()
    }
}

pub trait ToByteSlice<'a> {
    fn to_byte_slice(self) -> &'a [u8];
}
impl<'a, T: NoUninit> ToByteSlice<'a> for &'a [T] {
    fn to_byte_slice(self) -> &'a [u8] {
        cast_slice(self)
    }
}

macro_rules! seek_to {
    ($reader:expr, $pos:expr) => {
        $reader
            .seek(SeekFrom::Start($pos as u64))
            .map(|_| ())
            .map_err(|_| format!("Couldn't seek to {}", $pos))
    };
}
pub(crate) use seek_to;

pub fn nullpad_string(mut str: String, to_len: usize) -> String {
    let len = str.len();
    if len < to_len {
        str.push_str(&"\0".repeat(to_len - len));
    }
    str
}
