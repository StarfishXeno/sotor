use std::{
    fmt::Debug,
    fs::File,
    io::{self, prelude::*},
    mem::size_of,
    str::{self, Utf8Error},
};
use time::{macros::datetime, OffsetDateTime};

pub const DWORD_SIZE: usize = 4;

// read the whole file into a buffer
pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::with_capacity(file.metadata()?.len() as usize);
    file.read_to_end(&mut buf)?;

    Ok(buf)
}

// write the whole buffer into a file
pub fn write_file(path: &str, buf: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(buf)?;

    Ok(())
}

// turns &[u8] into a u32, f64, etc.
pub fn cast_bytes<T: ToBytes<SIZE>, const SIZE: usize>(bytes: &[u8]) -> T {
    let source: [u8; SIZE] = bytes[..SIZE].try_into().unwrap();
    T::from_le_bytes(source)
}

// this sucks, SIZE can't be deduced in current version of rust and you'd have to manually specify it every call
// fn sized_bytes_to_bytes<T, const SIZE: usize, E: Debug>(bytes: &[u8]) -> &[u8] where T: ToBytes<SIZE> + TryInto<usize, Error = E> {
//     let size: T = cast_bytes(&bytes);
//     let size = size.try_into().unwrap();
//     &bytes[SIZE..SIZE + size]
// }

// reads the size prefix of $type and returns that many following bytes
macro_rules! sized_bytes_to_bytes {
    ($bytes:expr, $type:ident) => {{
        let size_size = std::mem::size_of::<$type>();
        let size: $type = cast_bytes(&$bytes[..size_size]);
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

// reads <count> bytes into a buffer
pub fn read_bytes<T: Read + Seek>(reader: &mut T, count: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0; count];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}
// reads <count> chunks of <chunk_size> into a buffer
pub fn read_chunks<T: Read + Seek>(
    reader: &mut T,
    count: usize,
    chunk_size: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let buf = read_bytes(reader, count * chunk_size)?;
    let result = buf.chunks_exact(chunk_size).map(Into::into).collect();

    Ok(result)
}
// reads <count> DWORDs into a buffer
pub fn read_dwords<T: Read + Seek>(reader: &mut T, count: usize) -> io::Result<Vec<u32>> {
    Ok(read_chunks(reader, count, DWORD_SIZE)?
        .into_iter()
        .map(|c| cast_bytes(&c))
        .collect())
}

pub fn bytes_to_string(value: &[u8]) -> Result<String, Utf8Error> {
    str::from_utf8(value).map(str::to_owned)
}

// turns numerics (u16, f32, etc) into a u32, T can't be more than 4 bytes
#[allow(clippy::needless_pass_by_value)]
pub fn num_to_dword<T: ToBytes<SIZE>, const SIZE: usize>(num: T) -> u32 {
    assert!(SIZE <= 4, "T can't be larger than 4 bytes");
    let mut buf = [0u8; 4];
    for (idx, byte) in num.to_le_bytes().into_iter().enumerate() {
        buf[idx] = byte;
    }
    u32::from_ne_bytes(buf)
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

pub trait ToUSizeVec {
    fn to_usize_vec(self) -> Vec<usize>;
}
impl<T, E> ToUSizeVec for &[T]
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

impl_to_bytes!(u8 i8 u16 i16 u32 i32 u64 i64 f32 f64);

pub fn array_to_bytes<T: ToBytes<SIZE>, const SIZE: usize>(data: &[T]) -> Vec<u8> {
    let vec: Vec<u8> = data.iter().flat_map(T::to_le_bytes).collect();
    vec
}
macro_rules! seek_to {
    ($reader:expr, $pos:expr, $format_macro:ident) => {
        $reader
            .seek(SeekFrom::Start($pos as u64))
            .map(|_| ())
            .map_err(|_| $format_macro!("Couldn't seek to {}", $pos))
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

// years since 1900 and days since jan 1
pub fn get_erf_date() -> (u32, u32) {
    let now = OffsetDateTime::now_utc();
    let past = datetime!(1900 - 01 - 01 0:00 UTC);
    let build_year = now.year() - past.year();

    past.replace_year(now.year()).unwrap();
    let build_day = (now - past).whole_days();

    (build_year as u32, build_day as u32)
}
