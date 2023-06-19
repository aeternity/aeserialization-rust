use num_traits::ToPrimitive;
use crate::{error, Bytes};

// TODO: should I keep this as usize or change to u8 ?
const UNTAGGED_SIZE_LIMIT: usize = 55;
const UNTAGGED_LIMIT: u8 = 127;
const BYTE_ARRAY_OFFSET: u8 = 128;
const LIST_OFFSET: u8 = 192;

#[derive(Debug, Clone, PartialEq)]
pub enum RLPItem {
    ByteArray(Bytes),
    List(Vec<RLPItem>)
}

#[derive(Debug, PartialEq)]
pub enum DecodingErr {
    Trailing {
        input: Bytes,
        undecoded: Bytes,
        decoded: RLPItem
    },
    LeadingZerosInSize
}

pub trait ToRLPItem {
    fn to_rlp_item(&self) -> RLPItem;
}

pub trait FromRLPItem: Sized {
    fn from_rlp_item(item: &RLPItem) -> Result<Self, error::DecodingErr>;
}

pub fn encode(item: &RLPItem) -> Bytes {
    match item {
        RLPItem::ByteArray(bytes) =>
            if bytes.len() == 1 && bytes[0] <= UNTAGGED_LIMIT {
                bytes.to_vec()
            } else {
                add_size(BYTE_ARRAY_OFFSET, bytes.to_vec())
            }
        RLPItem::List(items) => {
            let bytes: Bytes = items.into_iter().flat_map(encode).collect();
            add_size(LIST_OFFSET, bytes)
        }
    }
}

pub fn decode(bytes: &Bytes) -> Result<RLPItem, DecodingErr> {
    // TODO: handle the case of empty bytes
    match try_decode(bytes)? {
        (item, []) => Ok(item),
        (item, rest) =>
            Err(DecodingErr::Trailing {
                input: bytes.to_vec(),
                undecoded: rest.to_vec(),
                decoded: item
            })
    }
}

fn try_decode(bytes: &[u8]) -> Result<(RLPItem, &[u8]), DecodingErr> {
    let res = match bytes[0] {
        0..=127 =>
            (RLPItem::ByteArray(bytes[0..1].to_vec()), &bytes[1..]),
        128..=183 => {
            let len: usize = bytes[0] as usize - 128;
            // TODO: Make sure that there is enough bytes
            (RLPItem::ByteArray(bytes[1..len + 1].to_vec()), &bytes[len + 1..])
        },
        184..=191 => {
            let len_bytes: usize = bytes[0] as usize - 183;
            // TODO: Make sure len_bytes > 0 && <= 8
            // TODO: Make sure that there is enough bytes
            // TOOD: Remove the unwrap and try_into
            // TODO: Make sure len does not start with 0 byte
            if bytes[1] == 0 {
                Err(DecodingErr::LeadingZerosInSize)?
            } else {
                let len: usize = bytes_to_size(bytes[1..len_bytes + 1].to_vec());
                (RLPItem::ByteArray(bytes[len_bytes + 1..len_bytes + len + 1].to_vec()), &bytes[len_bytes + len + 1..])
            }
        },
        192..=247 => {
            let len: usize = bytes[0] as usize - 192;
            let rest = &bytes[len + 1..];
            let mut list_rest = &bytes[1..len + 1];
            let mut items = Vec::with_capacity(len);
            while !list_rest.is_empty() {
                let decoded = try_decode(&list_rest)?;
                let item = decoded.0;
                list_rest = decoded.1;
                items.push(item);
            }
            items.truncate(items.len());
            (RLPItem::List(items), rest)
        },
        248..=255 => {
            let len_bytes: usize = bytes[0] as usize - 247;
            if bytes[1] == 0 {
                Err(DecodingErr::LeadingZerosInSize)?
            } else {
                let len: usize = bytes_to_size(bytes[1..len_bytes + 1].to_vec());

                let rest = &bytes[1 + len_bytes + len..];
                let mut list_rest = &bytes[1 + len_bytes..1 + len_bytes + len];
                let mut items = Vec::with_capacity(len);
                while !list_rest.is_empty() {
                    let decoded = try_decode(&list_rest)?;
                    let item = decoded.0;
                    list_rest = decoded.1;
                    items.push(item);
                }
                items.truncate(items.len());
                (RLPItem::List(items), rest)
            }
        }
    };

    Ok(res)
}

fn add_size(offset: u8, bytes: Bytes) -> Bytes {
    if bytes.len() <= UNTAGGED_SIZE_LIMIT {
        let mut res = Vec::with_capacity(bytes.len() + 1);
        res.push(offset + bytes.len() as u8);
        res.extend(bytes);
        res
    } else {
        let size_bytes = drop_leading_bytes(bytes.len().to_be_bytes().to_vec());
        let tagged_size = (UNTAGGED_SIZE_LIMIT + offset as usize + size_bytes.len())
            .to_u8()
            .expect("Large tagged size");

        let mut res = Vec::with_capacity(bytes.len() + 5);
        res.push(tagged_size);
        res.extend(size_bytes);
        res.extend(bytes);
        res
    }
}

fn bytes_to_size(mut bytes: Bytes) -> usize {
    let total = std::mem::size_of::<usize>();

    bytes.reverse();
    bytes.resize(total, 0);
    bytes.reverse();

    usize::from_be_bytes(bytes.try_into().unwrap())
}

fn size_to_min_bytes(size: usize) -> Bytes {
    size.to_be_bytes().iter().skip_while(|n| **n == 0).copied().collect()
}

// TODO: Remove this
fn drop_leading_bytes(bytes: Bytes) -> Bytes {
    bytes.iter().skip_while(|n| **n == 0).copied().collect()
}

impl ToRLPItem for u32 {
    fn to_rlp_item(&self) -> RLPItem {
        RLPItem::ByteArray(drop_leading_bytes(self.to_be_bytes().to_vec()))
    }
}

impl ToRLPItem for bool {
    fn to_rlp_item(&self) -> RLPItem {
        RLPItem::ByteArray(vec![*self as u8])
    }
}

impl ToRLPItem for Vec<u8> {
    fn to_rlp_item(&self) -> RLPItem {
        RLPItem::ByteArray(self.to_vec())
    }
}

// TODO: should this be removed?
impl ToRLPItem for Vec<RLPItem> {
    fn to_rlp_item(&self) -> RLPItem {
        RLPItem::List(self.to_vec())
    }
}

impl FromRLPItem for u32 {
    fn from_rlp_item(item: &RLPItem) -> Result<Self, error::DecodingErr> {
        match item {
            RLPItem::List(_) => Err(error::DecodingErr::InvalidInt),
            RLPItem::ByteArray(bytes) =>
                if bytes.len() > 0 && bytes.len() <= 4 && bytes[0] != 0 {
                    let mut bytes_vec = vec![0; 4 - bytes.len()];
                    bytes_vec.extend(bytes);
                    let bytes_arr: [u8; 4] = bytes_vec.try_into().or(Err(error::DecodingErr::InvalidInt))?;
                    Ok(u32::from_be_bytes(bytes_arr))
                } else {
                    Err(error::DecodingErr::InvalidInt)
                }

        }
    }
}

impl FromRLPItem for bool {
    fn from_rlp_item(item: &RLPItem) -> Result<Self, error::DecodingErr> {
        match item {
            RLPItem::List(_) => Err(error::DecodingErr::InvalidBool),
            RLPItem::ByteArray(bytes) =>
                if *bytes == vec![0u8] {
                    Ok(false)
                } else if *bytes == vec![1u8] {
                    Ok(true)
                } else {
                    Err(error::DecodingErr::InvalidBool)
                }
        }
    }
}

impl FromRLPItem for Vec<u8> {
    fn from_rlp_item(item: &RLPItem) -> Result<Self, error::DecodingErr> {
        match item {
            RLPItem::List(_) => Err(error::DecodingErr::InvalidBinary),
            RLPItem::ByteArray(bytes) => Ok(bytes.to_vec())
        }
    }
}

impl FromRLPItem for Vec<RLPItem> {
    fn from_rlp_item(item: &RLPItem) -> Result<Self, error::DecodingErr> {
        match item {
            RLPItem::ByteArray(_) => Err(error::DecodingErr::InvalidList),
            RLPItem::List(items) => Ok(items.to_vec())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn encode_then_decode(input: RLPItem, expect: Bytes) {
        let encoded = encode(&input);
        println!("{:?}", encoded);
        let decoded = decode(&encoded);
        println!("{:?}", decoded);

        assert_eq!(encoded, expect);
        assert_eq!(decoded, Ok(input));
    }

    #[test]
    fn one_byte() {
        let input = RLPItem::ByteArray(vec![42]);
        let expect = vec![42];
        encode_then_decode(input, expect);
    }

    #[test]
    fn another_one_byte() {
        let input = RLPItem::ByteArray(vec![127]);
        let expect = vec![127];
        encode_then_decode(input, expect);
    }

    #[test]
    fn zero_bytes() {
        let input = RLPItem::ByteArray(vec![]);
        let expect = vec![BYTE_ARRAY_OFFSET];
        encode_then_decode(input, expect);
    }

    #[test]
    fn two_bytes() {
        let input = RLPItem::ByteArray(vec![128]);
        let expect = vec![BYTE_ARRAY_OFFSET + 1, 128];
        encode_then_decode(input, expect);
    }

    #[test]
    fn one_byte_size_bytes() {
        let len = 55;
        let input_bytes: Bytes = (1..=len).collect();
        let input = RLPItem::ByteArray(input_bytes.to_vec());
        let expect= vec![BYTE_ARRAY_OFFSET + len]
            .into_iter()
            .chain(input_bytes)
            .collect();
        encode_then_decode(input, expect);
    }

    #[test]
    fn tagged_size_one_byte_bytes() {
        let len = 56;
        let tag = BYTE_ARRAY_OFFSET + UNTAGGED_SIZE_LIMIT as u8 + 1;
        let input_bytes = vec![42; len];
        let input = RLPItem::ByteArray(input_bytes.to_vec());
        let expect = vec![tag]
            .into_iter()
            .chain(size_to_min_bytes(len))
            .chain(input_bytes)
            .collect();
        encode_then_decode(input, expect);
    }

    #[test]
    fn tagged_size_two_bytes_bytes() {
        let len = 256;
        let len_bytes = 2;
        let tag = BYTE_ARRAY_OFFSET + UNTAGGED_SIZE_LIMIT as u8 + len_bytes;
        let input_bytes = vec![42; len];
        let input = RLPItem::ByteArray(input_bytes.to_vec());
        let expect = vec![tag]
            .into_iter()
            .chain(size_to_min_bytes(len))
            .chain(input_bytes)
            .collect();
        encode_then_decode(input, expect);
    }

    #[test]
    fn zero_bytes_list() {
        let input = RLPItem::List(vec![]);
        let expect = vec![LIST_OFFSET];
        encode_then_decode(input, expect);
    }

    #[test]
    fn one_byte_list() {
        let len = 1;
        let tag = LIST_OFFSET + len;
        let input_bytes = vec![42];
        let input = RLPItem::List(vec![input_bytes.to_rlp_item()]);
        let expect= vec![tag]
            .into_iter()
            .chain(input_bytes)
            .collect();
        encode_then_decode(input, expect);
    }

    #[test]
    fn byte_array_list() {
        let len = 55;
        let tag = LIST_OFFSET + len as u8;
        let input_nums = vec![42; len];
        let input_bytes: Vec<u8> = input_nums.iter().map(|x| *x as u8).collect();
        let byte_arrays: Vec<RLPItem> = input_nums.iter().map(|x| x.to_rlp_item()).collect();

        let input = RLPItem::List(byte_arrays);
        let expect= vec![tag]
            .into_iter()
            .chain(input_bytes)
            .collect();
        encode_then_decode(input, expect);
    }

    #[test]
    fn byte_array_tagged_size_one_byte_list() {
        let len = 56;
        let len_bytes = 1;
        let tag = LIST_OFFSET + UNTAGGED_SIZE_LIMIT as u8 + len_bytes;
        let input_nums = vec![42; len];
        let input_bytes: Vec<u8> = input_nums.iter().map(|x| *x as u8).collect();
        let byte_arrays: Vec<RLPItem> = input_nums.iter().map(|x| x.to_rlp_item()).collect();

        let input = RLPItem::List(byte_arrays);
        let expect= vec![tag]
            .into_iter()
            .chain(size_to_min_bytes(len))
            .chain(input_bytes)
            .collect();
        encode_then_decode(input, expect);
    }

    #[test]
    fn byte_array_tagged_size_two_bytes_list() {
        let len = 256;
        let len_bytes = 2;
        let tag = LIST_OFFSET + UNTAGGED_SIZE_LIMIT as u8 + len_bytes;
        let input_nums = vec![42; len];
        let input_bytes: Vec<u8> = input_nums.iter().map(|x| *x as u8).collect();
        let byte_arrays: Vec<RLPItem> = input_nums.iter().map(|x| x.to_rlp_item()).collect();

        let input = RLPItem::List(byte_arrays);
        let expect= vec![tag]
            .into_iter()
            .chain(size_to_min_bytes(len))
            .chain(input_bytes)
            .collect();
        encode_then_decode(input, expect);
    }

    #[test]
    fn gal_size_encoding_list() {
        let len = 56;
        let len_bytes = 1;
        let tag = LIST_OFFSET + UNTAGGED_SIZE_LIMIT as u8 + len_bytes;
        let input_nums = vec![42; len];
        let input_bytes: Vec<u8> = input_nums.iter().map(|x| *x as u8).collect();

        let input = vec![tag + 1]
            .into_iter()
            .chain(vec![0])
            .chain(size_to_min_bytes(len))
            .chain(input_bytes)
            .collect();
        assert_eq!(decode(&input), Err(DecodingErr::LeadingZerosInSize));
    }

    #[test]
    fn gal_size_encoding_byte_array() {
        let len = 256;
        let len_bytes = 2;
        let tag = BYTE_ARRAY_OFFSET + UNTAGGED_SIZE_LIMIT as u8 + len_bytes;
        let input_bytes = vec![42; len];
        let input = vec![tag + 1]
            .into_iter()
            .chain(vec![0])
            .chain(size_to_min_bytes(len))
            .chain(input_bytes)
            .collect();
        assert_eq!(decode(&input), Err(DecodingErr::LeadingZerosInSize));
    }
}