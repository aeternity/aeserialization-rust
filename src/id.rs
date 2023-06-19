use crate::{error::DecodingErr, rlp::{RLPItem, ToRLPItem, FromRLPItem}, Bytes};

use num::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use ts_rs::TS;

pub const PUB_SIZE: usize = 32;
pub const TAG_SIZE: usize = 1;
pub const SERIALIZED_SIZE: usize = TAG_SIZE + PUB_SIZE;

#[derive(Clone, Copy, FromPrimitive, ToPrimitive, TS)]
#[ts(export)]
pub enum Tag {
    Account = 1,
    Name = 2,
    Commitment = 3,
    Oracle = 4,
    Contract = 5,
    Channel = 6
}

pub type EncodedId = [u8; SERIALIZED_SIZE];

#[derive(Clone, Copy, TS)]
#[ts(export)]
pub struct Id {
    pub tag: Tag,
    pub val: [u8; 32]
}

impl ToRLPItem for Id {
    fn to_rlp_item(&self) -> RLPItem {
        let mut encoded: Bytes = Vec::with_capacity(33);
        encoded[0] = self.tag.to_u8().expect("id::Tag enum does not fit in u8");
        encoded[TAG_SIZE..].clone_from_slice(&self.val);
        RLPItem::ByteArray(encoded)
    }
}

impl FromRLPItem for Id {
    fn from_rlp_item(item: &RLPItem) -> Result<Self, DecodingErr> {
        match item {
            RLPItem::List(_) => Err(DecodingErr::InvalidId),
            RLPItem::ByteArray(bytes) => {
                let tag: Tag = Tag::from_u8(bytes[0]).ok_or(DecodingErr::InvalidId)?;
                let val: [u8; 32] = bytes[TAG_SIZE..].try_into().or(Err(DecodingErr::InvalidId))?;
                Ok(Id {tag, val})
            }
        }
    }
}
