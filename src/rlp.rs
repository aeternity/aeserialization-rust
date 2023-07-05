use crate::{error, Bytes};
use num_traits::ToPrimitive;

/// Max single-byte size description
const UNTAGGED_SIZE_LIMIT: u8 = 55;
/// Max single-byte value
const UNTAGGED_LIMIT: u8 = 127;
/// Threshold after which a byte array is described. For example, 128 indicates beginning of an
/// empty array, while 130 a two element array.
const BYTE_ARRAY_OFFSET: u8 = 128;
/// /// Threshold after which a list of RLP elements is described. For example, 200 indicates
/// beginning of an empty list, while a list elements of which are encoded with 8 bytes in total.
const LIST_OFFSET: u8 = 192;

// Pattern helpers
/// Max byte array indicator with a single-byte size description.
const BYTE_ARRAY_UNTAGGED_LIMIT: u8 = BYTE_ARRAY_OFFSET + UNTAGGED_SIZE_LIMIT;
/// Min byte array indicator with a multi-byte size description.
const BYTE_ARRAY_TAGGED_OFFSET: u8 = BYTE_ARRAY_UNTAGGED_LIMIT + 1;
/// Max byte array indicator.
const BYTE_ARRAY_LIMIT: u8 = LIST_OFFSET - 1;
/// Max byte list indicator with a single-byte size description.
const LIST_UNTAGGED_LIMIT: u8 = LIST_OFFSET + UNTAGGED_SIZE_LIMIT;
/// Max byte list indicator with a multi-byte size description.
const LIST_TAGGED_OFFSET: u8 = LIST_UNTAGGED_LIMIT + 1;

/// A recursive-length-prefix--enocded value. See the
/// [protocol](https://github.com/aeternity/protocol/blob/master/serializations.md#rlp-encoding) for
/// detailed description.
#[derive(Debug, Clone, PartialEq)]
pub enum RlpItem {
    ByteArray(Bytes),
    List(Vec<RlpItem>),
}

impl RlpItem {
    /// Unpack as a byte array.
    pub fn byte_array(&self) -> Result<Bytes, error::DecodingErr> {
        match self {
            RlpItem::ByteArray(arr) => Ok(arr.to_vec()),
            RlpItem::List(_) => Err(error::DecodingErr::InvalidBinary),
        }
    }

    /// Unpack as a list of Rlp items.
    pub fn list(&self) -> Result<Vec<RlpItem>, error::DecodingErr> {
        match self {
            RlpItem::ByteArray(_) => Err(error::DecodingErr::InvalidList),
            RlpItem::List(l) => Ok(l.to_vec()),
        }
    }

    /// Serializes an [RlpItem] into bytes.
    pub fn serialize(&self) -> Bytes {
        match self {
            RlpItem::ByteArray(bytes) => {
                if bytes.len() == 1 && bytes[0] <= UNTAGGED_LIMIT {
                    bytes.to_vec()
                } else {
                    Self::add_size(BYTE_ARRAY_OFFSET, bytes.to_vec())
                }
            }
            RlpItem::List(items) => {
                let bytes: Bytes = items.iter().flat_map(|x| x.serialize()).collect();
                Self::add_size(LIST_OFFSET, bytes)
            }
        }
    }

    /// Deserializes an [RlpItem]. Requires consuming the entire input.
    pub fn deserialize(bytes: &[u8]) -> Result<RlpItem, DecodingErr> {
        if bytes.is_empty() {
            Err(DecodingErr::Empty)?;
        }

        match Self::try_deserialize(bytes)? {
            (item, []) => Ok(item),
            (item, rest) => Err(DecodingErr::Trailing {
                input: bytes.to_vec(),
                undecoded: rest.to_vec(),
                decoded: item,
            }),
        }
    }

    /// Deserializes an [RlpItem]. Returns trailing input which was not consumed.
    pub fn try_deserialize(bytes: &[u8]) -> Result<(RlpItem, &[u8]), DecodingErr> {
        Self::try_decode_at(bytes, 0)
    }

    fn try_decode_at(bytes: &[u8], at: usize) -> Result<(RlpItem, &[u8]), DecodingErr> {
        let res = match bytes[0] {
            ..=UNTAGGED_LIMIT => (RlpItem::ByteArray(bytes[0..1].to_vec()), &bytes[1..]),
            BYTE_ARRAY_OFFSET..=BYTE_ARRAY_UNTAGGED_LIMIT => {
                let len: usize = (bytes[0] - BYTE_ARRAY_OFFSET) as usize;

                if bytes.len() < len + 1 {
                    Err(DecodingErr::SizeOverflow {
                        position: at,
                        expected: len,
                        actual: bytes.len(),
                    })?
                }

                (
                    RlpItem::ByteArray(bytes[1..len + 1].to_vec()),
                    &bytes[len + 1..],
                )
            }
            BYTE_ARRAY_TAGGED_OFFSET..=BYTE_ARRAY_LIMIT => {
                let len_bytes: usize = (bytes[0] - BYTE_ARRAY_UNTAGGED_LIMIT) as usize;

                if bytes.len() < len_bytes + 1 {
                    Err(DecodingErr::SizeOverflow {
                        position: at,
                        expected: len_bytes,
                        actual: bytes.len(),
                    })?
                }

                if bytes[1] == 0 {
                    Err(DecodingErr::LeadingZerosInSize { position: at + 1 })?
                }

                let len: usize = bytes_to_size(bytes[1..len_bytes + 1].to_vec());
                (
                    RlpItem::ByteArray(bytes[len_bytes + 1..len_bytes + len + 1].to_vec()),
                    &bytes[len_bytes + len + 1..],
                )
            }
            LIST_OFFSET..=LIST_UNTAGGED_LIMIT => {
                let len: usize = (bytes[0] - LIST_OFFSET) as usize;
                let rest = &bytes[len + 1..];
                let list_bytes = &bytes[1..len + 1];
                let items = Self::decode_list_at(list_bytes, at + 1)?;
                (RlpItem::List(items), rest)
            }
            LIST_TAGGED_OFFSET.. => {
                let len_bytes: usize = (bytes[0] - LIST_UNTAGGED_LIMIT) as usize;
                if bytes[1] == 0 {
                    Err(DecodingErr::LeadingZerosInSize { position: at + 1 })?
                }

                let len: usize = bytes_to_size(bytes[1..len_bytes + 1].to_vec());
                let rest = &bytes[1 + len_bytes + len..];
                let list_bytes = &bytes[1 + len_bytes..1 + len_bytes + len];

                let items = Self::decode_list_at(list_bytes, at + 1)?;
                (RlpItem::List(items), rest)
            }
        };

        Ok(res)
    }

    fn decode_list_at(mut bytes: &[u8], mut at: usize) -> Result<Vec<RlpItem>, DecodingErr> {
        let mut items = Vec::new();
        while !bytes.is_empty() {
            let (item, rest) = Self::try_decode_at(bytes, at)?;
            items.push(item);
            at += (bytes.len() + 1) - rest.len();
            bytes = rest;
        }
        Ok(items)
    }

    fn add_size(offset: u8, bytes: Bytes) -> Bytes {
        if bytes.len() <= UNTAGGED_SIZE_LIMIT as usize {
            let mut res = Vec::with_capacity(bytes.len() + 1);
            res.push(offset + bytes.len() as u8);
            res.extend(bytes);
            res
        } else {
            let size_bytes = usize_to_min_be_bytes(bytes.len());
            let tagged_size = (UNTAGGED_SIZE_LIMIT as usize + offset as usize + size_bytes.len())
                .to_u8()
                .expect("Large tagged size");

            let mut res = Vec::with_capacity(bytes.len() + 5);
            res.push(tagged_size);
            res.extend(size_bytes);
            res.extend(bytes);
            res
        }
    }
}

fn bytes_to_size(mut bytes: Bytes) -> usize {
    let total = std::mem::size_of::<usize>();

    bytes.reverse();
    bytes.resize(total, 0);
    bytes.reverse();

    usize::from_be_bytes(bytes.try_into().unwrap())
}

fn usize_to_min_be_bytes(n: usize) -> Bytes {
    let byte_len = n.ilog(256) as usize + 1;
    let bytes = n.to_be_bytes();
    bytes[bytes.len() - byte_len..].to_vec()
}

/// An RLP decoding error.
#[derive(Debug, PartialEq)]
pub enum DecodingErr {
    /// Expected to consume entire input, but there is data left.
    Trailing {
        input: Bytes,
        undecoded: Bytes,
        decoded: RlpItem,
    },
    /// Tagged size has trailing zero-bytes.
    LeadingZerosInSize { position: usize },
    /// Expected to read [expected] number of bytes, but the input is capped at [actual].
    SizeOverflow {
        position: usize,
        expected: usize,
        actual: usize,
    },
    /// Empty input.
    Empty,
}

/// Conversion to an RLP value.
pub trait ToRlpItem {
    fn to_rlp_item(&self) -> RlpItem;

    fn serialize_rlp(&self) -> Bytes {
        self.to_rlp_item().serialize()
    }
}

impl From<&dyn ToRlpItem> for RlpItem {
    fn from(item: &dyn ToRlpItem) -> Self {
        item.to_rlp_item()
    }
}

/// Conversion from an RLP value.
pub trait FromRlpItem: Sized {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, error::DecodingErr>;

    fn deserialize_rlp(data: &[u8]) -> Result<Self, error::DecodingErr> {
        let rlp = RlpItem::deserialize(data)
            .map_err(|_| error::DecodingErr::InvalidRlp)?;
        FromRlpItem::from_rlp_item(&rlp)
    }
}

impl ToRlpItem for RlpItem {
    fn to_rlp_item(&self) -> RlpItem {
        self.clone()
    }
}

impl ToRlpItem for u8 {
    fn to_rlp_item(&self) -> RlpItem {
        RlpItem::ByteArray(usize_to_min_be_bytes(*self as usize))
    }
}

impl ToRlpItem for u32 {
    fn to_rlp_item(&self) -> RlpItem {
        RlpItem::ByteArray(usize_to_min_be_bytes(*self as usize))
    }
}

impl ToRlpItem for bool {
    fn to_rlp_item(&self) -> RlpItem {
        RlpItem::ByteArray(vec![*self as u8])
    }
}

impl<T: ToRlpItem> ToRlpItem for Vec<T> {
    fn to_rlp_item(&self) -> RlpItem {
        RlpItem::List(self.iter().map(|x| x.to_rlp_item()).collect())
    }
}

impl<T: ToRlpItem> ToRlpItem for [T] {
    fn to_rlp_item(&self) -> RlpItem {
        RlpItem::List(self.iter().map(|x| x.to_rlp_item()).collect())
    }
}

impl FromRlpItem for RlpItem {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, error::DecodingErr> {
        Ok(item.clone())
    }
}

impl FromRlpItem for u8 {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, error::DecodingErr> {
        let bytes = item.byte_array()?;

        if bytes.len() != 1 {
            Err(error::DecodingErr::InvalidInt)?;
        }

        Ok(bytes[0])
    }
}

impl FromRlpItem for u32 {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, error::DecodingErr> {
        let bytes = item.byte_array()?;
        let size = std::mem::size_of::<Self>();

        if bytes.is_empty() || bytes.len() > size || (bytes.len() > 1 && bytes[0] == 0) {
            Err(error::DecodingErr::InvalidInt)?;
        }

        let mut bytes_vec = vec![0; size - bytes.len()];
        bytes_vec.extend(bytes);

        let bytes_arr = bytes_vec
            .try_into()
            .or(Err(error::DecodingErr::InvalidInt))?;

        Ok(Self::from_be_bytes(bytes_arr))
    }
}

impl FromRlpItem for bool {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, error::DecodingErr> {
        let bytes = item.byte_array()?;

        if *bytes == vec![0u8] {
            Ok(false)
        } else if *bytes == vec![1u8] {
            Ok(true)
        } else {
            Err(error::DecodingErr::InvalidBool)
        }
    }
}

impl<T: FromRlpItem> FromRlpItem for Vec<T> {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, error::DecodingErr> {
        let rlps = item.list()?;

        rlps.into_iter().map(|x| T::from_rlp_item(&x)).collect()
    }
}

mod erlang {
    use super::*;
    use rustler::*;

    fn make_bin<'a>(env: Env<'a>, data: &[u8]) -> Term<'a> {
        let mut bin = NewBinary::new(env, data.len());
        bin.as_mut_slice().copy_from_slice(data);
        Term::from(bin)
    }

    impl Encoder for RlpItem {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            match self {
                RlpItem::ByteArray(bytes) => make_bin(env, bytes),
                RlpItem::List(rlps) => rlps.iter().rfold(Term::list_new_empty(env), |acc, el| {
                    acc.list_prepend(el.encode(env))
                }),
            }
        }
    }

    impl<'a> Decoder<'a> for RlpItem {
        fn decode(term: Term) -> NifResult<RlpItem> {
            if term.is_binary() {
                Ok(RlpItem::ByteArray(
                    term.decode_as_binary()?.as_slice().to_vec(),
                ))
            } else if term.is_list() {
                let list: Vec<Term> = term.decode()?;
                let rlps: NifResult<Vec<RlpItem>> = list.iter().map(|x| x.decode()).collect();
                Ok(RlpItem::List(rlps?))
            } else {
                Err(Error::BadArg)
            }
        }
    }

    impl Encoder for DecodingErr {
        fn encode<'a>(self: &DecodingErr, env: Env<'a>) -> Term<'a> {
            match self {
                DecodingErr::Trailing {
                    input,
                    undecoded,
                    decoded,
                } => {
                    let header = Atom::from_str(env, "trailing").unwrap();
                    (header, input, undecoded, decoded).encode(env)
                }
                DecodingErr::LeadingZerosInSize { position } => {
                    let header = Atom::from_str(env, "leading_zeros_in_size").unwrap();
                    (header, position).encode(env)
                }
                DecodingErr::SizeOverflow {
                    position,
                    expected,
                    actual,
                } => {
                    let header = Atom::from_str(env, "size_overflow").unwrap();
                    (header, position, expected, actual).encode(env)
                }
                DecodingErr::Empty => Atom::from_str(env, "empty").unwrap().encode(env),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use prop::collection::vec;
    use proptest::{collection::VecStrategy, prelude::*};

    fn any_u8vec<TMin, TMax>(min_len: TMin, max_len: TMax) -> VecStrategy<proptest::num::u8::Any>
    where
        TMin: Into<usize>,
        TMax: Into<usize>,
    {
        vec(any::<u8>(), min_len.into()..max_len.into())
    }

    impl proptest::arbitrary::Arbitrary for RlpItem {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let leaf = any::<Vec<u8>>().prop_map(RlpItem::ByteArray);
            leaf.prop_recursive(
                5,    // deep
                256,  // max nodes
                1000, // max items per collection
                |inner| vec(inner, 0..10000).prop_map(RlpItem::List),
            )
            .boxed()
        }
    }

    fn encode_then_decode(input: RlpItem, expect: Bytes) {
        let encoded = input.serialize();
        let decoded = RlpItem::deserialize(&encoded);

        assert_eq!(encoded, expect);
        assert_eq!(decoded, Ok(input));
    }

    proptest! {
        #[test]
        fn usize_to_min_be_bytes_plain(n: usize) {
            let expected: Vec<u8> = n.to_be_bytes().iter().skip_while(|n| **n == 0).copied().collect();
            let actual   = usize_to_min_be_bytes(n);
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn encode_decode(rlp: RlpItem) {
            let e = rlp.serialize();
            let d = RlpItem::deserialize(&e).expect("decoding failed");
            prop_assert_eq!(rlp, d);
        }

        #[test]
        fn one_byte(b in 0..=UNTAGGED_LIMIT) {
            let input = RlpItem::ByteArray(vec![b]);
            let expect = vec![b];
            encode_then_decode(input, expect);
        }


        #[test]
        fn one_byte_size_bytes(input_bytes in any_u8vec(1u8, UNTAGGED_SIZE_LIMIT + 1)) {
            prop_assume!(input_bytes[0] > UNTAGGED_LIMIT);

            let input  = RlpItem::ByteArray(input_bytes.to_vec());
            let expect = vec![BYTE_ARRAY_OFFSET + input_bytes.len() as u8]
                .into_iter()
                .chain(input_bytes)
                .collect();
            encode_then_decode(input, expect);
        }

        #[test]
        fn tagged_size_bytes(input_bytes in any_u8vec(UNTAGGED_SIZE_LIMIT + 1, UNTAGGED_SIZE_LIMIT as usize * 8)) {
            let len = input_bytes.len();
            let len_bytes = len.ilog(256) as u8 + 1;
            let tag = BYTE_ARRAY_OFFSET + UNTAGGED_SIZE_LIMIT + len_bytes;
            let input = RlpItem::ByteArray(input_bytes.to_vec());
            let expect = vec![tag]
                .into_iter()
                .chain(usize_to_min_be_bytes(len))
                .chain(input_bytes)
                .collect();
            encode_then_decode(input, expect);
        }

        #[test]
        fn one_byte_array_list(
            input_list
                in vec(any::<u8>().prop_map(|n| RlpItem::ByteArray(vec![n % (UNTAGGED_SIZE_LIMIT + 1)])),
                       1..=UNTAGGED_SIZE_LIMIT as usize
        )) {
            let payload: Bytes = input_list.iter().flat_map(|x| x.serialize()).collect();
            let tag = LIST_OFFSET + payload.len() as u8;
            let input = RlpItem::List(input_list);
            let expect= vec![tag]
                .into_iter()
                .chain(payload)
                .collect();
            encode_then_decode(input, expect);
        }

        #[test]
        fn byte_array_tagged_size_list(
            input_list
                in vec(any::<u8>().prop_map(|n| RlpItem::ByteArray(vec![n % (UNTAGGED_SIZE_LIMIT + 1)])),
                       (LIST_OFFSET as usize + 1)..=(UNTAGGED_SIZE_LIMIT as usize * 4))
        ) {
            let payload: Bytes = input_list.iter().flat_map(|x| x.serialize()).collect();
            let len = payload.len();
            let len_bytes = len.ilog(256) as u8 + 1;
            let tag = LIST_OFFSET + UNTAGGED_SIZE_LIMIT + len_bytes;

            let input = RlpItem::List(input_list);
            let expect= vec![tag]
                .into_iter()
                .chain(usize_to_min_be_bytes(len))
                .chain(payload)
                .collect();
            encode_then_decode(input, expect);
        }

    }

    #[test]
    fn zero_bytes() {
        let input = RlpItem::ByteArray(vec![]);
        let expect = vec![BYTE_ARRAY_OFFSET];
        encode_then_decode(input, expect);
    }

    #[test]
    fn zero_bytes_list() {
        let input = RlpItem::List(vec![]);
        let expect = vec![LIST_OFFSET];
        encode_then_decode(input, expect);
    }

    #[test]
    fn gal_size_encoding_list() {
        let len = 56;
        let len_bytes = 1;
        let tag = LIST_UNTAGGED_LIMIT + len_bytes;
        let input_nums = vec![42; len];
        let input_bytes: Vec<u8> = input_nums.iter().map(|x| *x as u8).collect();

        let input: Bytes = vec![tag + 1]
            .into_iter()
            .chain(vec![0])
            .chain(usize_to_min_be_bytes(len))
            .chain(input_bytes)
            .collect();
        assert_eq!(
            RlpItem::deserialize(&input),
            Err(DecodingErr::LeadingZerosInSize { position: 1 })
        );
    }

    #[test]
    fn gal_size_encoding_byte_array() {
        let len = 256;
        let len_bytes = 2;
        let tag = BYTE_ARRAY_UNTAGGED_LIMIT + len_bytes;
        let input_bytes = vec![42; len];
        let input: Bytes = vec![tag + 1]
            .into_iter()
            .chain(vec![0])
            .chain(usize_to_min_be_bytes(len))
            .chain(input_bytes)
            .collect();
        assert_eq!(
            RlpItem::deserialize(&input),
            Err(DecodingErr::LeadingZerosInSize { position: 1 })
        );
    }
}
