use std::{
    fs::File,
    io::BufReader,
    io::{self, prelude::*},
    str::{self, Utf8Error},
};

pub const DWORD_SIZE: usize = 4;

// turns &[u8] into a u32, f64, etc.
macro_rules! cast_bytes {
    ($bytes:expr, $type:ident) => {{
        const SIZE: usize = std::mem::size_of::<$type>();
        let source: [u8; SIZE] = $bytes[..SIZE].try_into().unwrap();
        $type::from_le_bytes(source)
    }};
}

// for some reason this macro is cursed and can't be referenced without the full module qualifier 
// TODO figure out why?
// reads the size prefix of $type and returns that many following bytes
macro_rules! bytes_to_sized_bytes {
    ($bytes:expr, $type:ident) => {{
        let size_size = std::mem::size_of::<$type>();
        let size = cast_bytes!($bytes[..size_size], $type) as usize;
        &$bytes[size_size..size_size + size]
    }};
}

// turns &[u8] with a size prefix into a string
macro_rules! bytes_to_exo_string {
    ($bytes:expr, $type:ident) => {{
        let source = crate::util::bytes_to_sized_bytes!($bytes, $type);
        bytes_to_string(source)
    }};
}

pub(crate) use bytes_to_exo_string;
pub(crate) use bytes_to_sized_bytes;
pub(crate) use cast_bytes;


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
        .map(|c| cast_bytes!(c, u32))
        .collect())
}

pub fn bytes_to_string(value: &[u8]) -> Result<String, Utf8Error> {
    str::from_utf8(value).map(|str| str.to_owned())
}
