use crate::{error::DecodingErr, rlp::{RLPItem, ToRLPItem, FromRLPItem}, Bytes};

use num::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum Tag {
    Account = 1,
    Name,
    Commitment,
    Oracle,
    Contract,
    Channel
}

type EncodedId = [u8; 33];

#[derive(Clone, Copy)]
pub struct Id {
    tag: Tag,
    val: [u8; 32]
}

impl ToRLPItem for Id {
    fn to_rlp_item(&self) -> RLPItem {
        let mut encoded: Bytes = Vec::with_capacity(33);
        encoded[0] = self.tag.to_u8().expect("id::Tag enum does not fit in u8");
        encoded[1..].clone_from_slice(&self.val);
        RLPItem::ByteArray(encoded)
    }
}

impl FromRLPItem for Id {
    fn from_rlp_item(item: &RLPItem) -> Result<Self, DecodingErr> {
        match item {
            RLPItem::List(_) => Err(DecodingErr::InvalidId),
            RLPItem::ByteArray(bytes) => {
                let tag: Tag = Tag::from_u8(bytes[0]).ok_or(DecodingErr::InvalidId)?;
                let val: [u8; 32] = bytes[1..].try_into().or(Err(DecodingErr::InvalidId))?;
                Ok(Id {tag, val})
            }
        }
    }
}
