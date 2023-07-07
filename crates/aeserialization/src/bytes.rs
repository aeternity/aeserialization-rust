use crate::{rlp, error::DecodingErr};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Bytes {
    data: Vec<u8>,
}

impl Bytes {
    pub fn from(data: &[u8]) -> Self {
        Bytes {
            data: data.to_vec(),
        }
    }

    pub fn new() -> Self {
        Bytes { data: Vec::new() }
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn reverse(&mut self) {
        self.data.reverse()
    }

    pub fn extend(&mut self, data: &[u8]) {
        self.data.extend(data)
    }

    pub fn resize(&mut self, size: usize, fill: u8) {
        self.data.resize(size, fill)
    }
}

impl std::ops::Index<usize> for Bytes {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.data[idx]
    }
}

impl std::ops::Index<core::ops::Range<usize>> for Bytes {
    type Output = [u8];
    fn index(&self, idx: core::ops::Range<usize>) -> &Self::Output {
        &self.data[idx]
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl FromIterator<u8> for Bytes {
    #[inline]
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        Bytes {
            data: Vec::<u8>::from_iter(iter),
        }
    }
}

impl IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = <Vec<u8> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl rlp::ToRlpItem for Bytes {
    fn to_rlp_item(&self) -> rlp::RlpItem {
        rlp::RlpItem::ByteArray(self.clone())
    }
}

impl rlp::FromRlpItem for Bytes {
    fn from_rlp_item(rlp: &rlp::RlpItem) -> Result<Bytes, DecodingErr> {
        let arr = rlp.byte_array()?;
        Ok(arr)
    }
}

mod erlang {
    use super::*;
    use rustler::*;

    impl Encoder for Bytes {
        fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
            let mut bin = types::binary::NewBinary::new(env, self.data.len());
            let data = bin.as_mut_slice();
            data.copy_from_slice(&self.data);

            Binary::from(bin).to_term(env)
        }
    }

    impl<'a> Decoder<'a> for Bytes {
        fn decode(term: Term<'a>) -> NifResult<Bytes> {
            if !term.is_binary() {
                Err(Error::BadArg)?;
            }

            let bin = Binary::from_term(term)?;
            let data = bin.as_slice();
            Ok(Bytes {
                data: data.to_vec(),
            })
        }
    }
}
