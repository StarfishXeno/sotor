use crate::formats::FileHead;
use bytemuck::{bytes_of, cast_slice, pod_read_unaligned, AnyBitPattern, NoUninit, Pod};
use std::{
    fmt::Debug,
    io::{self, prelude::*, Cursor, Read},
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
    T: TryInto<usize, Error = E> + Copy,
    E: Debug,
{
    fn to_usize_vec(self) -> Vec<usize> {
        self.iter().map(|i| (*i).try_into().unwrap()).collect()
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
            .map_err(|_| format!("couldn't seek to {}", $pos))
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

pub fn take_slice<'a, T: Pod + Debug>(input: &'a mut Cursor<&[u8]>, len: usize) -> Option<&'a [T]> {
    let offset = input.position() as usize;
    let next_offset = offset + len * size_of::<T>();
    let buf = *input.get_ref();
    let Some(bytes) = buf.get(offset..next_offset) else {
        return None;
    };
    input.set_position(next_offset as u64);

    Some(cast_slice(bytes))
}

pub fn take<T: Pod>(input: &mut Cursor<&[u8]>) -> Option<T> {
    let offset = input.position() as usize;
    let next_offset = offset + size_of::<T>();
    let buf = *input.get_ref();
    let v = pod_read_unaligned(buf.get(offset..next_offset)?);
    input.set_position(next_offset as u64);

    Some(v)
}

pub fn take_string(input: &mut Cursor<&[u8]>, len: usize) -> Option<String> {
    let Some(bytes) = take_slice(input, len) else {
        return None;
    };
    Some(bytes_to_string(bytes))
}

pub fn take_string_until(input: &mut Cursor<&[u8]>, terminator: u8) -> Option<String> {
    let mut buf = vec![];
    input.read_until(terminator, &mut buf).ok()?;

    if let Some(byte) = buf.pop() {
        if byte != terminator {
            return None;
        }
    } else {
        return None;
    }

    String::from_utf8(buf).ok()
}

pub fn take_head(input: &mut Cursor<&[u8]>) -> Option<FileHead> {
    let head = take_string(input, 8)?;
    let (tp, version) = head.split_at(4);

    Some(FileHead::new(tp.to_owned(), version.to_owned()))
}
