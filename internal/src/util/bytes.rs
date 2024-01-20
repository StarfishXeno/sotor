use crate::formats::FileHead;
use bytemuck::{
    bytes_of, cast_slice, checked::try_cast_slice, try_pod_read_unaligned, AnyBitPattern, NoUninit,
    Pod,
};
use std::{borrow::Cow, fmt::Debug, io::prelude::*, mem::size_of};

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

pub fn bytes_to_string(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

pub trait IntoUsizeVec {
    fn into_usize_vec(self) -> Vec<usize>;
}
impl<T, E> IntoUsizeVec for &[T]
where
    T: TryInto<usize, Error = E> + Copy,
    E: Debug,
{
    fn into_usize_vec(self) -> Vec<usize> {
        self.iter().map(|i| (*i).try_into().unwrap()).collect()
    }
}

pub trait IntoUsizeArray<const SIZE: usize> {
    fn into_usize_array(self) -> [usize; SIZE];
}
impl<T, E, const SIZE: usize> IntoUsizeArray<SIZE> for [T; SIZE]
where
    T: TryInto<usize, Error = E> + Copy,
    E: Debug,
{
    fn into_usize_array(self) -> [usize; SIZE] {
        self.map(|i| i.try_into().unwrap())
    }
}

pub trait IntoByteSlice<'a> {
    fn into_byte_slice(self) -> &'a [u8];
}
impl<'a, T: NoUninit> IntoByteSlice<'a> for &'a [T] {
    fn into_byte_slice(self) -> &'a [u8] {
        cast_slice(self)
    }
}

macro_rules! seek_to {
    ($reader:expr, $pos:expr) => {
        $reader
            .seek(SeekFrom::Start($pos as u64))
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

pub type Cursor<'a> = std::io::Cursor<&'a [u8]>;

pub fn take_bytes<'a>(input: &mut Cursor<'a>, len: usize) -> Option<&'a [u8]> {
    let pos = input.position() as usize;
    let end = pos + len;
    input.consume(len);
    input.get_ref().get(pos..end)
}

pub fn take_slice<'a, T: Pod>(input: &'a mut Cursor, len: usize) -> Option<Cow<'a, [T]>> {
    let byte_len = len * size_of::<T>();
    let bytes = take_bytes(input, byte_len)?;
    if let Ok(slice) = try_cast_slice(bytes) {
        Some(Cow::Borrowed(slice))
    } else {
        // pointer allignment issues won't let us use bytemuck's cast_slice
        // have to copy everything over into a properly aligned vec
        let mut vec = Vec::<T>::with_capacity(len);
        let ptr = vec.as_mut_ptr().cast::<u8>();
        unsafe {
            ptr.copy_from_nonoverlapping(bytes.as_ptr(), byte_len);
            vec.set_len(len);
        }
        Some(Cow::Owned(vec))
    }
}

pub fn take_slice_sized<'a, P: AnyBitPattern + TryInto<usize>, T: Pod>(
    input: &'a mut Cursor,
) -> Option<Cow<'a, [T]>> {
    let size = take::<P>(input)?;
    take_slice(input, size.try_into().ok()?)
}

pub fn take<T: AnyBitPattern>(input: &mut Cursor) -> Option<T> {
    let bytes = take_bytes(input, size_of::<T>())?;
    try_pod_read_unaligned(bytes).ok()
}

pub fn take_string(input: &mut Cursor, len: usize) -> Option<String> {
    let bytes = take_bytes(input, len)?;
    Some(bytes_to_string(bytes.to_vec()))
}

pub fn take_string_sized<P: AnyBitPattern + TryInto<usize>>(input: &mut Cursor) -> Option<String> {
    let bytes = take_slice_sized::<P, u8>(input)?;
    Some(bytes_to_string(bytes.into_owned()))
}

pub fn take_string_until(input: &mut Cursor, terminator: u8) -> Option<String> {
    let mut buf = vec![];
    input.read_until(terminator, &mut buf).ok()?;

    let last = buf.pop();
    if last.is_none() || last.unwrap() != terminator {
        return None;
    }

    Some(bytes_to_string(buf))
}

pub fn take_string_trimmed(input: &mut Cursor, max_len: usize) -> Option<String> {
    let bytes = take_bytes(input, max_len)?;
    let mut limited = Cursor::new(bytes);
    let mut buf = Vec::with_capacity(max_len);
    limited.read_until(b'\0', &mut buf).ok()?;
    if Some(b'\0') == buf.last().copied() {
        buf.pop();
    };
    buf.shrink_to_fit();
    Some(bytes_to_string(buf))
}

pub fn take_head(input: &mut Cursor) -> Option<FileHead> {
    let head = take_string(input, 8)?;
    let (tp, version) = head.split_at(4);

    Some((tp, version).into())
}
