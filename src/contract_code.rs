use crate::rlp::{self, ToRlpItem, RlpItem, FromRlpItem};
use crate::error::DecodingErr;
use crate::Bytes;
use crate::Field;

#[derive(Debug, PartialEq)]
// #[derive(rustler::NifMap)]
struct TypeInfo {
    type_hash: Bytes,
    name: Bytes,
    payable: bool,
    arg_type: Bytes,
    out_type: Bytes
}

impl ToRlpItem for TypeInfo {
    fn to_rlp_item(&self) -> RlpItem {
        RlpItem::List(vec![RlpItem::List(vec![
            self.type_hash.to_rlp_item(),
            self.name.to_rlp_item(),
            self.payable.to_rlp_item(),
            self.arg_type.to_rlp_item(),
            self.out_type.to_rlp_item()
        ])])
    }
}

impl FromRlpItem for TypeInfo {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, DecodingErr> {
        match item {
            RlpItem::List(list) =>
                match list.first() {
                    Some(RlpItem::List(items)) =>
                        if items.len() == 5 {
                            Ok(TypeInfo {
                                type_hash: Bytes::from_rlp_item(&items[0])?,
                                name: Bytes::from_rlp_item(&items[1])?,
                                payable: bool::from_rlp_item(&items[2])?,
                                arg_type: Bytes::from_rlp_item(&items[3])?,
                                out_type: Bytes::from_rlp_item(&items[4])?
                            })
                        } else {
                            Err(DecodingErr::InvalidList)
                        }
                    _ => Err(DecodingErr::InvalidList),
                }
            _ => Err(DecodingErr::InvalidList),
        }
    }
}

#[derive(Debug, PartialEq)]
// #[derive(rustler::NifMap)]
pub struct Code {
    type_info: Vec<TypeInfo>,
    byte_code: Bytes,
    source_hash: Bytes,
    compiler_version: Bytes,
    payable: bool
}

pub fn hash_source_code(str: &str) -> Bytes {
    use blake2::{Blake2b, Digest, digest::consts::U32};
    type Blake2b32 = Blake2b<U32>;
    let mut hasher = Blake2b32::new();
    hasher.update(str);
    hasher.finalize().to_vec()
}

pub fn serialize(code: &Code) -> Bytes {
    let fields = [
        Field {
            name: "tag".to_string(),
            val: 70u32.to_rlp_item() // TODO: should not be hardcoded
        },
        Field {
            name: "vsn".to_string(),
            val: 3u32.to_rlp_item() // TODO: should this be hardcoded?
        },
        Field {
            name: "source_hash".to_string(),
            val: code.source_hash.to_rlp_item()
        },
        Field {
            name: "type_info".to_string(),
            val: RlpItem::List(vec![]) // code.type_info.to_rlp_item()
        },
        Field {
            name: "byte_code".to_string(),
            val: code.byte_code.to_rlp_item()
        },
        Field {
            name: "compiler_version".to_string(),
            val: code.compiler_version.to_rlp_item()
        },
        Field {
            name: "payable".to_string(),
            val: code.payable.to_rlp_item()
        }
    ];

    let items: Vec<RlpItem> = fields.into_iter().map(|f| f.val).collect();
    rlp::encode(&items.to_rlp_item())
}

pub fn deserialize(bytes: &Vec<u8>) -> Result<Code, DecodingErr> {
    let deser = match rlp::decode(&bytes) {
        Ok(RlpItem::List(items)) =>
            Code {
                source_hash: Bytes::from_rlp_item(&items[2])?,
                type_info: vec![], //TypeInfo::from_rlp_item(&items[3])?,
                byte_code: Bytes::from_rlp_item(&items[4])?,
                compiler_version: Bytes::from_rlp_item(&items[5])?,
                payable: bool::from_rlp_item(&items[6])?
            },
        _ => Err(DecodingErr::InvalidRlp)?
    };
    Ok(deser)
}

mod erlang {
    use rustler::*;

    mod fields {
        rustler::atoms! {
            type_hash,
            name,
            payable,
            arg_type,
            out_type,
            type_info,
            byte_code,
            source_hash,
            compiler_version,
        }
    }

    fn make_bin<'a>(env: Env<'a>, data: &crate::Bytes) -> Term<'a> {
        let mut bin = NewBinary::new(env, data.len());
        bin.as_mut_slice().copy_from_slice(&data);
        Binary::from(bin).to_term(env)
    }

    fn open_bin<'a>(term: Term<'a>) -> NifResult<crate::Bytes> {
         if !term.is_binary() {
            Err(Error::BadArg)?;
        }

        let bin = Binary::from_term(term)?;
        let data = bin.as_slice();
        Ok(data.to_vec())
    }

    impl Encoder for crate::contract_code::TypeInfo {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            Term::map_from_pairs(
                env,
                &[(fields::type_hash(), make_bin(env, &self.type_hash)),
                  (fields::name(), make_bin(env, &self.name)),
                  (fields::payable(), self.payable.encode(env)),
                  (fields::arg_type(), make_bin(env, &self.arg_type)),
                  (fields::out_type(), make_bin(env, &self.out_type)),
                ]
            ).expect("Failed creating an Erlang map")
        }
    }

    impl<'a> Decoder<'a> for crate::contract_code::TypeInfo {
        fn decode(term: Term<'a>) -> NifResult<Self> {
            let type_info = crate::contract_code::TypeInfo{
                type_hash: open_bin(term.map_get(fields::type_hash())?)?,
                name: open_bin(term.map_get(fields::name())?)?,
                payable: term.map_get(fields::payable())?.decode()?,
                arg_type: open_bin(term.map_get(fields::arg_type())?)?,
                out_type: open_bin(term.map_get(fields::out_type())?)?
            };
            Ok(type_info)
        }
    }

    impl Encoder for crate::contract_code::Code {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            Term::map_from_pairs(
                env,
                &[(fields::type_info(), self.type_info.encode(env)),
                  (fields::byte_code(), make_bin(env, &self.byte_code)),
                  (fields::payable(), self.payable.encode(env)),
                  (fields::source_hash(), make_bin(env, &self.source_hash)),
                  (fields::compiler_version(), make_bin(env, &self.compiler_version)),
                ]
            ).expect("Failed creating an Erlang map")
        }
    }

    impl<'a> Decoder<'a> for crate::contract_code::Code {
        fn decode(term: Term<'a>) -> NifResult<Self> {
            let code = crate::contract_code::Code{
                type_info: term.map_get(fields::type_info())?.decode()?,
                byte_code: open_bin(term.map_get(fields::byte_code())?)?,
                payable: term.map_get(fields::payable())?.decode()?,
                source_hash: open_bin(term.map_get(fields::source_hash())?)?,
                compiler_version: open_bin(term.map_get(fields::compiler_version())?)?
            };
            Ok(code)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn source_hash() {
        let source = "contract Foo = ...";
        let expect = vec![
            48,58,125,237,188,44,120,213,52,155,92,4,213,8,157,236,198,161,240,
            9,117,91,60,167,64,44,67,82,145,174,238,243
        ];

        assert_eq!(hash_source_code(&source), expect);
    }

    #[test]
    fn sophia_contract_version3_serialize() {
        let input = Code {
            byte_code: "DUMMY CODE".as_bytes().to_vec(),
            type_info: TypeInfo {
                type_hash: vec![],
                name: vec![],
                payable: false,
                arg_type: vec![],
                out_type: vec![]
            },
            source_hash: hash_source_code("contract Foo = ..."),
            compiler_version: "3.1.4".as_bytes().to_vec(),
            payable: true
        };
        let expect = vec![
            248,60,70,3,160,48,58,125,237,188,44,120,213,52,155,92,4,213,8,157,
            236,198,161,240,9,117,91,60,167,64,44,67,82,145,174,238,243,198,197,
            128,128,0,128,128,138,68,85,77,77,89,32,67,79,68,69,133,51,46,49,46,52,1
        ];

        let serialized = serialize(&input);
        let deserialized = deserialize(&serialized);

        assert_eq!(serialized, expect);
        assert_eq!(deserialized, Ok(input));
    }
}
