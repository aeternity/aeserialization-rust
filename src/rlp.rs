use std::mem;

const UNTAGGED_SIZE_LIMIT: u8 = 55;
const UNTAGGED_LIMIT: u8 = 127;
const BYTE_ARRAY_OFFSET: u8 = 128;
const LIST_OFFSET: u8 = 192;

pub fn encode(x: &[u8]) -> Vec<u8> {
    if x.len() == 0 || (x.len() == 1 && x[0] <= UNTAGGED_LIMIT) {
        return x.to_vec();
    } else {
        return add_size(BYTE_ARRAY_OFFSET as usize, x);
    }
}

pub fn encode_many<'a, I>(xs: I) -> Vec<u8>
where
    I: IntoIterator<Item = &'a [u8]>,
{
    return add_size(LIST_OFFSET as usize, &xs.into_iter().flat_map(encode).collect::<Vec<u8>>());
}

fn encode_usize(x: usize) -> Vec<u8> {
    let byte_len = x.ilog(256) as usize + 1;
    let idx = mem::size_of_val(&x) - byte_len;
    Vec::from(&x.to_be_bytes()[idx..])
}

fn add_size(offset: usize, x: &[u8]) -> Vec<u8> {
    let size: usize = x.len();
    let size_bin: &[u8] = &encode_usize(x.len());
    if size <= UNTAGGED_SIZE_LIMIT as usize {
        return vec![size_bin, x].concat();
    } else {
        let tagged_size = UNTAGGED_SIZE_LIMIT as usize + offset + size_bin.len();
        assert!(tagged_size < 256);
        return vec![&[tagged_size as u8], size_bin, x].concat();
    }
}

pub fn decode(x: &[u8]) -> Vec<u8> {
    x.to_vec()
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest!{
        // #[test]
        // fn encode_decode(b: Vec<u8>) {
        //     let b1 = decode(&encode(&b));
        //     prop_assert_eq!(b, b1);
        // }

        // #[test]
        // fn decode_encode(b: Vec<u8>) {
        //     let b1 = encode(&decode(&b));
        //     prop_assert_eq!(b, b1);
        // }

        // #[test]
        // fn one_byte_size_bytes_test(l: u8) {
        //     let s = BYTE_ARRAY_OFFSET + l;
        //     let x: Vec<u8> = (1..=l).collect();
        //     let e = encode(&x[..]);
        //     assert_eq!(&vec![&[s], &x[..]].concat(), &e);
        //     assert_eq!(x, decode(&e));
        // }
    }

    fn roundtrip(b: &[u8]) {
        let e = encode(b);
        let d = decode(&e);
        assert_eq!(b.to_vec(), d);
    }

    #[test]
    fn one_byte_test() {
        roundtrip(&[42])
    }

    #[test]
    fn another_one_byte_test() {
        roundtrip(&[127]);
    }

    #[test]
    fn zero_bytes_test() {
        assert_eq!(encode_usize(BYTE_ARRAY_OFFSET as usize), encode(&[]));
    }

    #[test]
    fn two_bytes_test() {
        assert_eq!(vec![BYTE_ARRAY_OFFSET + 1, 128], encode(&[128]));
    }

    #[test]
    fn one_byte_size_bytes_test() {
        let l = UNTAGGED_SIZE_LIMIT;
        let s = BYTE_ARRAY_OFFSET + l;
        let x: Vec<u8> = (1..=l).collect();
        let e = encode(&x[..]);
        assert_eq!(vec![&[s], &x[..]].concat(), e);
        assert_eq!(x, decode(&e));
    }

    #[test]
    fn tagged_size_one_byte_bytes_test() {
        let l = UNTAGGED_SIZE_LIMIT + 1;
        let tag = BYTE_ARRAY_OFFSET + (UNTAGGED_SIZE_LIMIT as u8) + 1;
        let x: Vec<u8> = std::iter::repeat(42).take(l as usize).collect();
        let e = encode(&x[..]);
        assert_eq!(&vec![&[tag, x.len() as u8], &x[..]].concat(), &e);
        assert_eq!(x, decode(&e));
    }

    #[test]
    fn tagged_size_two_bytes_bytes_test() {
        let l: usize = 256;
        let tag = BYTE_ARRAY_OFFSET + (UNTAGGED_SIZE_LIMIT as u8) + 2;
        let x: Vec<u8> = std::iter::repeat(42).take(l as usize).collect();
        let e = encode(&x[..]);
        let s = encode_usize(x.len());
        assert_eq!(&vec![&[tag], &s[..], &x[..]].concat(), &e);
        assert_eq!(x, decode(&e));
    }

    #[test]
    fn zero_bytes_list_test() {
        let x: Vec<u8> = vec![];
        assert_eq!(vec![LIST_OFFSET], encode_many([]));
        assert_eq!(x, decode(&[LIST_OFFSET]));
    }

    #[test]
    fn one_byte_list_test() {
        let l = 1;
        let tag = LIST_OFFSET + l;
        let x = std::iter::repeat(&[42 as u8] as &[u8]).take(l as usize);
        let e = encode_many(x.clone());
        assert_eq!(vec![tag, 42], e);
        assert_eq!(x.flatten().map(|n| *n).collect::<Vec<u8>>(), decode(&e));
    }

    #[test]
    fn byte_array_list_test() {
        let l = UNTAGGED_SIZE_LIMIT;
        let tag = LIST_OFFSET + l;
        let x = std::iter::repeat(&[42 as u8] as &[u8]).take(l as usize);
        let e = encode_many(x.clone());
        let y = x.flatten().map(|n| *n).collect::<Vec<_>>();
        assert_eq!(vec![&[tag], &y[..]].concat(), e);
        assert_eq!(y, decode(&e));
    }

//     #[test]
//     fn byte_array_tagged_size_one_byte_list_test() {
//         L = 56,
//         SizeSize = 1,
//         Tag = ?LIST_OFFSET + ?UNTAGGED_SIZE_LIMIT + SizeSize,
//         X = lists:duplicate(L, [42]),
//         Y = list_to_binary(X),
//         S = byte_size(Y),
//         E = [Tag, S:SizeSize/unit:8, Y/binary] = encode(X),
//         X = decode(E)
//     }

//     #[test]
//     fn byte_array_tagged_size_two_bytes_list_test() {
//         L = 256,
//         SizeSize = 2,
//         Tag = ?LIST_OFFSET + ?UNTAGGED_SIZE_LIMIT + SizeSize,
//         X = lists:duplicate(L, [42]),
//         Y = list_to_binary(X),
//         S = byte_size(Y),
//         E = [Tag, S:SizeSize/unit:8, Y/binary] = encode(X),
//         X = decode(E)
//     }

//     #[test]
//     fn illegal_size_encoding_list_test() {
//         // Ensure we start with somehting legal.
//         L = 56,
//         SizeSize = 1,
//         Tag = ?LIST_OFFSET + ?UNTAGGED_SIZE_LIMIT + SizeSize,
//         X = lists:duplicate(L, [42]),
//         Y = list_to_binary(X),
//         S = byte_size(Y),
//         E = [Tag, S:SizeSize/unit:8, Y/binary] = encode(X),
//         X = decode(E),

//         // Add leading zeroes to the size field.
//         E1 = [(Tag + 1), 0, S:SizeSize/unit:8, Y/binary],
//         ?assertError(leading_zeroes_in_size, decode(E1))
//     }

//                              #[test]
//                              fn illegal_size_encoding_byte_array_test() {
//                                  // Ensure we start with somehting legal.
//         L = 256,
//         SizeSize = 2,
//         Tag = ?BYTE_ARRAY_OFFSET + ?UNTAGGED_SIZE_LIMIT + SizeSize,
//         X = list_to_binary(lists:duplicate(L, 42)),
//         S = byte_size(X),
//         E = [Tag, S:SizeSize/unit:8, X/binary] = encode(X),
//         X = decode(E),

//         %% Add leading zeroes to the size field.
//         E1 = [(Tag + 1), 0, S:SizeSize/unit:8, X/binary],
//         ?assertError(leading_zeroes_in_size, decode(E1))

//                              }
}
