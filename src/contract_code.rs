use crate::error::DecodingErr;
use crate::rlp::{self, FromRlpItem, RlpItem, ToRlpItem};
use crate::Bytes;

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
}

impl Code {
    /// Serialize FATE code as a byte-encoded RLP object.
    pub fn serialize(self: &Code) -> Bytes {
        self.serialize_rlp()
    }

    /// Deserializes a byte-encoded RLP object into FATE code.
    pub fn deserialize(bytes: &Vec<u8>) -> Result<Code, DecodingErr> {
        rlp::FromRlpItem::deserialize_rlp(bytes)
    }
}

impl rlp::ToRlpItem for Code {
    fn to_rlp_item(&self) -> RlpItem {
        let fields = vec![
            // Tag
            CODE_TAG.to_rlp_item(), // TODO: should not be hardcoded
            // Compiler version
            VSN.to_rlp_item(), // TODO: should this be hardcoded?
            // Source hash
            rlp::RlpItem::ByteArray(self.source_hash.to_vec()),
            // Type info (AEVM residue, has to be empty)
            rlp::RlpItem::List(vec![]),
            // Byte code
            rlp::RlpItem::ByteArray(self.byte_code.to_vec()),
            // Contract version
            rlp::RlpItem::ByteArray(self.compiler_version.to_vec()),
            // Payable
            self.payable.to_rlp_item(),
        ];
        rlp::RlpItem::List(fields)
    }
}

impl FromRlpItem for Code {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, crate::error::DecodingErr> {
        let items = item.list().map_err(|_| DecodingErr::InvalidRlp)?;

        if items[3].list()?.len() > 0 {
            // This field is a residue after AEVM. In FATE it has to be an empty list.
            Err(DecodingErr::InvalidCode)?;
        }

        Ok(Code {
            source_hash: items[2].byte_array()?,
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

const CODE_TAG: u8 = 70;
const VSN: u8 = 3;

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

    impl Encoder for crate::contract_code::Code {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            Term::map_from_pairs(
                env,
                &[
                    (fields::type_info(), Term::list_new_empty(env)),
                    (fields::byte_code(), make_bin(env, &self.byte_code)),
                    (fields::payable(), self.payable.encode(env)),
                    (fields::source_hash(), make_bin(env, &self.source_hash)),
                    (
                        fields::compiler_version(),
                        make_bin(env, &self.compiler_version),
                    ),
                ],
            )
            .expect("Failed creating an Erlang map")
        }
    }

    impl<'a> Decoder<'a> for crate::contract_code::Code {
        fn decode(term: Term<'a>) -> NifResult<Self> {
            if !term.map_get(fields::type_info())?.is_empty_list() {
                Err(Error::BadArg)?;
            }

            let code = crate::contract_code::Code {
                byte_code: open_bin(term.map_get(fields::byte_code())?)?,
                payable: term.map_get(fields::payable())?.decode()?,
                source_hash: open_bin(term.map_get(fields::source_hash())?)?,
                compiler_version: open_bin(term.map_get(fields::compiler_version())?)?,
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

        assert_eq!(hash_source_code(&source), expect);
    }

    #[test]
    fn sophia_contract_version3_serialize() {
        let input = Code {
            byte_code: "DUMMY_CODE".as_bytes().to_vec(),
            source_hash: hash_source_code("contract Foo = ..."),
            compiler_version: "3.1.4".as_bytes().to_vec(),
            payable: true,
        };
        // Taken from the original Erlang implementation
        let expect = vec![
            246,70,3,160,48,58,125,237,188,44,120,213,52,155,92,4,213,8,157,236,198,161,
            240,9,117,91,60,167,64,44,67,82,145,174,238,243,192,138,68,85,77,77,89,95,67,
            79,68,69,133,51,46,49,46,52,1
        ];

        // TODO: figure out which is correct
        // let expect = vec![
        //     248,60,70,3,160,48,58,125,237,188,44,120,213,52,155,92,4,213,8,157,
        //     236,198,161,240,9,117,91,60,167,64,44,67,82,145,174,238,243,198,197,
        //     128,128,0,128,128,138,68,85,77,77,89,32,67,79,68,69,133,51,46,49,46,52,1];

        let serialized = input.serialize();
        let deserialized = Code::deserialize(&serialized);

        assert_eq!(serialized, expect);
        assert_eq!(deserialized, Ok(input));
    }
}
