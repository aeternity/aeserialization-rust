use crate::error::DecodingErr;
use crate::rlp::{FromRlpItem, RlpItem, ToRlpItem};
use crate::Bytes;

// TODO: this should come from another module which has not been rewritten yet
/// Identifier tag of serialized contract code
const CODE_TAG: u8 = 70;

/// Contract format version.
const VSN: u8 = 3;

#[derive(Debug, PartialEq)]
struct TypeInfo {
    type_hash: Bytes,
    name: Bytes,
    payable: bool,
    arg_type: Bytes,
    out_type: Bytes,
}

impl ToRlpItem for TypeInfo {
    fn to_rlp_item(&self) -> RlpItem {
        RlpItem::List(vec![RlpItem::List(vec![
            self.type_hash.to_rlp_item(),
            self.name.to_rlp_item(),
            self.payable.to_rlp_item(),
            self.arg_type.to_rlp_item(),
            self.out_type.to_rlp_item(),
        ])])
    }
}

impl FromRlpItem for TypeInfo {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, DecodingErr> {
        let items = item.list().map_err(|_| DecodingErr::InvalidRlp)?;

        if items.len() != 5 {
            Err(DecodingErr::InvalidRlp)?;
        }

        Ok(TypeInfo {
            type_hash: items[0].byte_array()?,
            name: items[1].byte_array()?,
            payable: bool::from_rlp_item(&items[2])?,
            arg_type: items[3].byte_array()?,
            out_type: items[4].byte_array()?,
        })
    }
}

/// FATE contract code with metadata
#[derive(Debug, PartialEq)]
pub struct Code {
    /// Byte code of the contract.
    pub byte_code: Bytes,
    /// Whether the contract can receive tokens through a `spend` transaction.
    pub payable: bool,
    /// Hash of the source code in the original smart contract language. Note that verification of
    /// this field is not imposed by the æternity protocol, thus its validity has to always be
    /// checked before the contract is used.
    pub source_hash: Bytes,
    /// Version of the compiler of the original smart contract language. Note that verification of
    /// this field is not imposed by the æternity protocol, thus its validity has to always be
    /// checked before the contract is used.
    pub compiler_version: Bytes,
    /// AEVM residue. Kept for compatibility.
    type_info: Vec<TypeInfo>,
}

impl Code {
    /// Serialize FATE code as a byte-encoded RLP object.
    pub fn serialize(self: &Code) -> Bytes {
        self.serialize_rlp()
    }

    /// Deserializes a byte-encoded RLP object into FATE code.
    pub fn deserialize(bytes: &[u8]) -> Result<Code, DecodingErr> {
        FromRlpItem::deserialize_rlp(bytes)
    }
}

impl ToRlpItem for Code {
    fn to_rlp_item(&self) -> RlpItem {
        let fields = vec![
            // Tag
            CODE_TAG.to_rlp_item(),
            // Contract version
            VSN.to_rlp_item(),
            // Source hash
            RlpItem::ByteArray(self.source_hash.to_vec()),
            // Type info (generally useless, AEVM residue)
            self.type_info.to_rlp_item(),
            // Byte code
            RlpItem::ByteArray(self.byte_code.to_vec()),
            // Contract version
            RlpItem::ByteArray(self.compiler_version.to_vec()),
            // Payable
            self.payable.to_rlp_item(),
        ];
        RlpItem::List(fields)
    }
}

impl FromRlpItem for Code {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, DecodingErr> {
        let items = item.list().map_err(|_| DecodingErr::InvalidRlp)?;

        if items.len() != 7 {
            Err(DecodingErr::InvalidRlp)?;
        }

        if u8::from_rlp_item(&items[0])? != CODE_TAG {
            Err(DecodingErr::InvalidRlp)?;
        }

        Ok(Code {
            source_hash: items[2].byte_array()?,
            type_info: Vec::<TypeInfo>::from_rlp_item(&items[3])?,
            byte_code: items[4].byte_array()?,
            compiler_version: items[5].byte_array()?,
            payable: bool::from_rlp_item(&items[6])?,
        })
    }
}

/// A universal function to hash original contract source code. Note that verifying the hash is not
/// imposed by the æternity protocol, thus its validity has to always be checked before the contract
/// is used.
pub fn hash_source_code(str: &str) -> Bytes {
    use blake2::{digest::consts::U32, Blake2b, Digest};
    type Blake2b32 = Blake2b<U32>;
    let mut hasher = Blake2b32::new();
    hasher.update(str);
    hasher.finalize().to_vec()
}

mod erlang {
    use rustler::*;
    use super::*;

    mod fields {
        rustler::atoms! {
            type_hash,
            name,
            payable,
            arg_type,
            out_type,
            byte_code,
            source_hash,
            compiler_version,
            type_info,
        }
    }

    fn make_bin<'a>(env: Env<'a>, data: &crate::Bytes) -> Term<'a> {
        let mut bin = NewBinary::new(env, data.len());
        bin.as_mut_slice().copy_from_slice(data);
        Binary::from(bin).to_term(env)
    }

    fn open_bin(term: Term) -> NifResult<crate::Bytes> {
        if !term.is_binary() {
            Err(Error::BadArg)?;
        }

        let bin = Binary::from_term(term)?;
        let data = bin.as_slice();
        Ok(data.to_vec())
    }

    impl Encoder for TypeInfo {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            (make_bin(env, &self.type_hash),
             make_bin(env, &self.name),
             self.payable.encode(env),
             make_bin(env, &self.arg_type),
             make_bin(env, &self.out_type),
            ).encode(env)
        }
    }

    impl<'a> Decoder<'a> for TypeInfo {
        fn decode(term: Term<'a>) -> NifResult<Self> {
            let (type_hash,
                 name,
                 payable_,
                 arg_type,
                 out_type,
            ) = <(Term<'a>, Term<'a>, bool, Term<'a>, Term<'a>) as Decoder<'a>>::decode(term)?;
            let type_info = TypeInfo {
                type_hash: open_bin(type_hash)?,
                name: open_bin(name)?,
                payable: payable_,
                arg_type: open_bin(arg_type)?,
                out_type: open_bin(out_type)?,
            };
            Ok(type_info)
        }
    }

    impl Encoder for Code {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            Term::map_from_pairs(
                env,
                &[
                    (fields::byte_code(), make_bin(env, &self.byte_code)),
                    (fields::payable(), self.payable.encode(env)),
                    (fields::source_hash(), make_bin(env, &self.source_hash)),
                    (
                        fields::compiler_version(),
                        make_bin(env, &self.compiler_version),
                    ),
                    (fields::type_info(), self.type_info.encode(env)),
                ],
            )
            .expect("Failed creating an Erlang map")
        }
    }

    impl<'a> Decoder<'a> for Code {
        fn decode(term: Term<'a>) -> NifResult<Self> {
            if !term.map_get(fields::type_info())?.is_empty_list() {
                Err(Error::BadArg)?;
            }

            let code = Code {
                byte_code: open_bin(term.map_get(fields::byte_code())?)?,
                payable: term.map_get(fields::payable())?.decode()?,
                source_hash: open_bin(term.map_get(fields::source_hash())?)?,
                compiler_version: open_bin(term.map_get(fields::compiler_version())?)?,
                type_info: term.map_get(fields::type_info())?.decode()?,
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
        // Taken from the original Erlang implementation
        let expect = vec![
            48, 58, 125, 237, 188, 44, 120, 213, 52, 155, 92, 4, 213, 8, 157, 236, 198, 161, 240,
            9, 117, 91, 60, 167, 64, 44, 67, 82, 145, 174, 238, 243,
        ];

        assert_eq!(hash_source_code(source), expect);
    }

    #[test]
    fn sophia_contract_version3_serialize() {
        let input = Code {
            byte_code: "DUMMY_CODE".as_bytes().to_vec(),
            source_hash: hash_source_code("contract Foo = ..."),
            compiler_version: "3.1.4".as_bytes().to_vec(),
            payable: true,
            type_info: vec![],
        };
        // Taken from the original Erlang implementation
        let expect = vec![
            246,70,3,160,48,58,125,237,188,44,120,213,52,155,92,4,213,8,157,236,198,161,
            240,9,117,91,60,167,64,44,67,82,145,174,238,243,192,138,68,85,77,77,89,95,67,
            79,68,69,133,51,46,49,46,52,1
        ];

        let serialized = input.serialize();
        let deserialized = Code::deserialize(&serialized);

        assert_eq!(serialized, expect);
        assert_eq!(deserialized, Ok(input));
    }


    #[test]
    fn sophia_contract_w_type_info_version3_serialize() {
        let type_info = TypeInfo {
            type_hash: vec![21, 37],
            name: vec![],
            payable: true,
            arg_type: vec![42, 0],
            out_type: vec![255, 7],
        };

        let input = Code {
            byte_code: "DUMMY_CODE".as_bytes().to_vec(),
            source_hash: hash_source_code("contract Foo = ..."),
            compiler_version: "3.1.4".as_bytes().to_vec(),
            payable: true,
            type_info: vec![type_info],
        };
        // Taken from the original Erlang implementation
        let expect = vec![
            248,66,70,3,160,48,58,125,237,188,44,120,213,52,
            155,92,4,213,8,157,236,198,161,240,9,117,91,60,
            167,64,44,67,82,145,174,238,243,204,203,130,21,
            37,128,1,130,42,0,130,255,7,138,68,85,77,77,89,
            32,67,79,68,69,133,51,46,49,46,52,1
        ];

        let serialized = input.serialize();
        let deserialized = Code::deserialize(&serialized);

        assert_eq!(serialized, expect);
        assert_eq!(deserialized, Ok(input));
    }
}
