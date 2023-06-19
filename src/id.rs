use crate::error::{EncodingErr, DecodingErr};

use num::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};
use ts_rs::TS;

pub const PUB_SIZE: usize = 32;
pub const TAG_SIZE: usize = 1;
pub const SERIALIZED_SIZE: usize = TAG_SIZE + PUB_SIZE;

#[derive(FromPrimitive, ToPrimitive, TS)]
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

#[derive(TS)]
#[ts(export)]
pub struct Id {
    pub tag: Tag,
    pub val: [u8; 32]
}

pub fn encode(id: &Id) -> Result<EncodedId, EncodingErr> {
    let mut encoded: EncodedId = [0; SERIALIZED_SIZE];
    encoded[0] = id.tag.to_u8().ok_or(EncodingErr::InvalidId)?;
    encoded[TAG_SIZE..].clone_from_slice(&id.val);
    Ok(encoded)
}

pub fn decode(bin: &EncodedId) -> Result<Id, DecodingErr> {
    let tag: Tag = Tag::from_u8(bin[0]).ok_or(DecodingErr::InvalidId)?;
    let val: [u8; 32] = bin[TAG_SIZE..].try_into().map_err(|_e| DecodingErr::InvalidId)?;
    Ok(Id {tag, val})
}
