use crate::{
    formats::{
        key::{Key, KeyResRef},
        ResourceKey, ResourceType,
    },
    util::{bytes_to_string, SResult, ToUsizeVec},
};
use bytemuck::{cast_slice, try_pod_read_unaligned, Pod};
use std::{collections::HashMap, fmt::Debug, mem::size_of};

pub type Take<'a, T> = (T, &'a [u8]);

fn take_slice_at<T: Pod + Debug>(input: &[u8], offset: usize, len: usize) -> Option<Take<&[T]>> {
    let Some(bytes) = input.get(offset..offset + len * size_of::<T>()) else {
        return None;
    };

    Some((cast_slice(bytes), &input[len..]))
}

fn take_slice<T: Pod + Debug>(input: &[u8], len: usize) -> Option<Take<&[T]>> {
    take_slice_at(input, 0, len)
}

fn take_at<T: Pod>(input: &[u8], offset: usize) -> Option<Take<T>> {
    let v = try_pod_read_unaligned(&input[offset..offset + size_of::<T>()]).ok()?;
    Some((v, &input[offset..]))
}

fn take<T: Pod>(input: &[u8]) -> Option<Take<T>> {
    take_at(input, 0)
}

fn take_string_at(input: &[u8], offset: usize, len: usize) -> Option<Take<String>> {
    let Some((bytes, next)) = take_slice_at(input, offset, len) else {
        return None;
    };
    Some((bytes_to_string(bytes), next))
}

fn take_string(input: &[u8], len: usize) -> Option<Take<String>> {
    take_string_at(input, 0, len)
}

fn read_header(bytes: &[u8]) -> SResult<(usize, usize)> {
    let (file_head, next) = take_string(bytes, 8).ok_or("couldn't read file version and type")?;
    if file_head != "KEY V1  " {
        return Err(format!("invalid file type or version {file_head}"));
    };
    let dwords = take_slice::<u32>(next, 4)
        .ok_or("could read header contents")?
        .0;
    // don't care about the rest of the header
    let mut content = dwords.to_usize_vec().into_iter();
    let key_count = {
        content.next();
        content.next().unwrap()
    };
    let key_offset = {
        content.next();
        content.next().unwrap()
    };

    Ok((key_count, key_offset))
}

fn read_resources(
    bytes: &[u8],
    count: usize,
    offset: usize,
) -> SResult<HashMap<ResourceKey, KeyResRef>> {
    const RESOURCE_SIZE_BYTES: usize = 22;

    let key_table = take_slice_at::<u8>(bytes, offset, count * RESOURCE_SIZE_BYTES)
        .ok_or("couldn't read key table")?
        .0;
    let mut resources = HashMap::with_capacity(count);
    for key in key_table.chunks_exact(RESOURCE_SIZE_BYTES) {
        let (file_name, next) = take_string(key, 16).unwrap();
        let (tp_raw, next) = take::<u16>(next).unwrap();
        let Ok(tp) = ResourceType::try_from(tp_raw) else {
            // we don't care about most resource types and the relevant ones are defined
            continue;
        };
        let id = take::<u32>(next).unwrap().0;
        let file_idx = id >> 20;
        let resource_idx = (id >> 14) - (file_idx << 6);

        resources.insert(
            (file_name.trim_end_matches('\0').to_string(), tp).into(),
            KeyResRef {
                file_idx,
                resource_idx,
            },
        );
    }

    Ok(resources)
}

pub fn read(bytes: &[u8]) -> SResult<Key> {
    let (key_count, key_offset) = read_header(bytes)?;
    let resources = read_resources(bytes, key_count, key_offset)?;

    Ok(Key { resources })
}
