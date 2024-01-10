use crate::formats::FileHead;
use bytemuck::{bytes_of, cast_slice, pod_read_unaligned, AnyBitPattern, NoUninit, Pod};
use std::{
    fmt::Debug,
    io::{prelude::*, Cursor},
    mem::size_of,
};

pub const DWORD_SIZE: usize = 4;

// returns a buffer with <bytes> prefixed with it's length
pub fn bytes_to_sized_bytes<const SIZE_SIZE: usize>(bytes: &[u8]) -> Vec<u8> {
    let len = bytes.len();
    let mut buf = Vec::with_capacity(len + SIZE_SIZE);
    let len_bytes = len.to_le_bytes();
    buf.extend(&len_bytes[..SIZE_SIZE]);
    buf.extend(bytes);

    buf
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

pub fn take_slice<T: Pod + Debug>(input: &mut Cursor<&[u8]>, len: usize) -> Option<Vec<T>> {
    let offset = input.position() as usize;
    let size = size_of::<T>();
    let next_offset = offset + len * size;
    let buf = *input.get_ref();
    let bytes = buf.get(offset..next_offset)?;
    input.set_position(next_offset as u64);
    let mut vec = Vec::<T>::with_capacity(len);
    let ptr = vec.as_mut_ptr().cast::<u8>();
    // fuck you alignment
    // miri says it's ok
    unsafe {
        ptr.copy_from_nonoverlapping(bytes.as_ptr(), len * size);
        vec.set_len(len);
    }
    Some(vec)
}

pub fn take_slice_sized<P: AnyBitPattern + TryInto<usize>, T: Pod + Debug>(
    input: &mut Cursor<&[u8]>,
) -> Option<Vec<T>> {
    let size = take::<P>(input)?;
    take_slice(input, size.try_into().ok()?)
}

pub fn take<T: AnyBitPattern>(input: &mut Cursor<&[u8]>) -> Option<T> {
    let offset = input.position() as usize;
    let next_offset = offset + size_of::<T>();
    let buf = *input.get_ref();
    let v = pod_read_unaligned(buf.get(offset..next_offset)?);
    input.set_position(next_offset as u64);

    Some(v)
}

pub fn take_string(input: &mut Cursor<&[u8]>, len: usize) -> Option<String> {
    let bytes = take_slice(input, len)?;
    String::from_utf8(bytes).ok()
}

pub fn take_string_sized<P: AnyBitPattern + TryInto<usize>>(
    input: &mut Cursor<&[u8]>,
) -> Option<String> {
    let size = take::<P>(input)?;
    take_string(input, size.try_into().ok()?)
}

pub fn take_string_until(input: &mut Cursor<&[u8]>, terminator: u8) -> Option<String> {
    let mut buf = vec![];
    input.read_until(terminator, &mut buf).ok()?;

    let last = buf.pop();
    if last.is_none() || last.unwrap() != terminator {
        return None;
    }

    String::from_utf8(buf).ok()
}

pub fn take_string_trimmed(input: &mut Cursor<&[u8]>, max_len: usize) -> Option<String> {
    let position = input.position() as usize;
    let mut limited = Cursor::new(input.get_ref().get(position..position + max_len)?);
    let mut buf = Vec::with_capacity(max_len);
    limited.read_until(b'\0', &mut buf).ok()?;
    input.consume(max_len);

    String::from_utf8(buf).ok()
}

pub fn take_head(input: &mut Cursor<&[u8]>) -> Option<FileHead> {
    let head = take_string(input, 8)?;
    let (tp, version) = head.split_at(4);

    Some((tp, version).into())
}
