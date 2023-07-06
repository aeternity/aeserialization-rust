use crate::{error::DecodingErr, rlp::{RlpItem, ToRlpItem, FromRlpItem}, Bytes};

use num::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use ts_rs::TS;

/// Size of the id payload (eg, a public key).
pub const PUB_SIZE: usize = 32;
/// Size of the id type tag.
pub const TAG_SIZE: usize = 1;
/// Total byte size of a serialized id.
pub const SERIALIZED_SIZE: usize = TAG_SIZE + PUB_SIZE;

/// Denotes the type of an id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive, TS)]
#[ts(export)]
#[derive(rustler::NifTaggedEnum)]
pub enum Tag {
    Account = 1,
    Name = 2,
    Commitment = 3,
    Oracle = 4,
    Contract = 5,
    Channel = 6
}

/// Wrapper for an id payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TS)]
pub struct EncodedId { pub bytes: [u8; PUB_SIZE] } // TODO: hermetize

/// Identifier of a chain object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TS)]
#[ts(export)]
pub struct Id {
    pub tag: Tag,
    pub val: EncodedId
}

impl Id {
    /// Serializes an id into a byte array.
    pub fn serialize(&self) -> Bytes {
        let mut encoded: Bytes = vec![0; 33];
        encoded[0] = self.tag.to_u8().expect("id::Tag enum does not fit in u8");
        encoded[TAG_SIZE..].clone_from_slice(&self.val.bytes);
        encoded
    }

    /// Deserializes an id from a byte array.
    pub fn deserialize(bytes: &[u8]) -> Result<Id, DecodingErr> {
        if bytes.len() != SERIALIZED_SIZE {
            Err(DecodingErr::InvalidIdSize)?;
        }

        let tag: Tag = Tag::from_u8(bytes[0]).ok_or(DecodingErr::InvalidIdTag)?;
        let val: [u8; 32] = bytes[TAG_SIZE..].try_into().or(Err(DecodingErr::InvalidIdPub))?;
        Ok(Id {tag, val: EncodedId{bytes: val}})
    }
}

impl ToRlpItem for Id {
    fn to_rlp_item(&self) -> RlpItem {
        let encoded = self.serialize();
        RlpItem::ByteArray(encoded)
    }
}

impl FromRlpItem for Id {
    fn from_rlp_item(item: &RlpItem) -> Result<Self, DecodingErr> {
        match item {
            RlpItem::List(_) => Err(DecodingErr::InvalidRlp),
            RlpItem::ByteArray(bytes) => {
                Id::deserialize(bytes)
            }
        }
    }
}

mod erlang {
    use rustler::*;
    use crate::id::*;

    impl Encoder for EncodedId {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            let mut bin = NewBinary::new(env, self.bytes.len());
            bin.as_mut_slice().copy_from_slice(&self.bytes);
            Binary::from(bin).to_term(env)
        }
    }

    impl<'a> Decoder<'a> for EncodedId {
        fn decode(term: Term<'a>) -> NifResult<EncodedId> {
            let bin = term.decode_as_binary()?;
            let bytes: &[u8; 32] = bin
                .as_slice()
                .try_into()
                .map_err(|_| Error::BadArg)?;

            Ok(EncodedId {bytes: *bytes})
        }
    }

    impl Encoder for Id {
        fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
            (Atom::from_str(env, "id").unwrap(),
             self.tag,
             self.val
            ).encode(env)
        }
    }

    impl<'a> Decoder<'a> for Id {
        fn decode(term: Term<'a>) -> NifResult<Id> {
            let tup = types::tuple::get_tuple(term)?;

            if tup.len() != 3 {
                Err(Error::BadArg)?;
            }

            if tup[0].atom_to_string()? != "id" {
                Err(Error::BadArg)?;
            }

            Ok(Id{
                tag: tup[1].decode()?,
                val: Decoder::decode(tup[2])?
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    impl proptest::arbitrary::Arbitrary for Tag {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(Tag::Account),
                Just(Tag::Name),
                Just(Tag::Commitment),
                Just(Tag::Oracle),
                Just(Tag::Contract),
                Just(Tag::Channel),
            ].boxed()
        }
    }

    impl proptest::arbitrary::Arbitrary for Id {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Tag>(), any::<[u8; 32]>())
                .prop_map(|(t, v)| Id{tag: t, val: EncodedId{bytes: v}})
                .boxed()
        }
    }

    proptest! {
        #[test]
        fn id_rlp_roundtrip(id: Id) {
            let rlp = id.to_rlp_item();
            let id1: Id = FromRlpItem::from_rlp_item(&rlp).expect("Decoding id from rlp");
            prop_assert_eq!(id1, id);
        }
    }

}
