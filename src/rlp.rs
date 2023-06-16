use std::mem;

const UNTAGGED_SIZE_LIMIT: usize = 55;
const UNTAGGED_LIMIT: u8 = 127;
const BYTE_ARRAY_OFFSET: usize = 128;
const LIST_OFFSET: usize = 192;

pub fn encode(x: &[u8]) -> Vec<u8> {
    if x.len() == 0 || (x.len() == 1 && x[0] <= UNTAGGED_LIMIT) {
        return x.to_vec();
    } else {
        return add_size(BYTE_ARRAY_OFFSET, x);
    }
}

pub fn encode_many<'a, I>(xs: I) -> Vec<u8>
where
    I: IntoIterator<Item = &'a [u8]>,
{
    return xs.into_iter().flat_map(encode).collect::<Vec<u8>>();
}

fn add_size(offset: usize, x: &[u8]) -> Vec<u8> {
    let size: usize = x.len();
    let size_bin_size: usize = size.ilog(256) as usize + 1;
    let size_bin: &[u8] = &size.to_be_bytes()[(mem::size_of_val(&size) - size_bin_size)..];
    if size <= UNTAGGED_SIZE_LIMIT {
        return vec![size_bin, x].concat();
    } else {
        let tagged_size = UNTAGGED_SIZE_LIMIT + offset + (size_bin_size);
        assert!(tagged_size < 256);
        return vec![&[tagged_size as u8], size_bin, x].concat();
    }
}
