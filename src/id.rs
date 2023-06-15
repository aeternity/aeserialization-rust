use crate::error::{EncodingErr, DecodingErr};

use num::{FromPrimitive, ToPrimitive};
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(FromPrimitive, ToPrimitive)]
pub enum Tag {
    Account = 1,
    Name,
    Commitment,
    Oracle,
    Contract,
    Channel
}

type EncodedId = [u8; 33];

pub struct Id {
    tag: Tag,
    val: [u8; 32]
}

pub fn encode(id: &Id) -> Result<EncodedId, EncodingErr> {
    let mut encoded: EncodedId = [0; 33];
    encoded[0] = id.tag.to_u8().ok_or(EncodingErr::InvalidId)?;
    encoded[1..].clone_from_slice(&id.val);
    Ok(encoded)
}

pub fn decode(bin: &EncodedId) -> Result<Id, DecodingErr> {
    let tag: Tag = Tag::from_u8(bin[0]).ok_or(DecodingErr::InvalidId)?;
    let val: [u8; 32] = bin[1..].try_into().map_err(|_e| DecodingErr::InvalidId)?;
    Ok(Id {tag, val})
}
