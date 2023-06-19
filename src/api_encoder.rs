use crate::id;
use crate::Bytes;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KnownType {
    KeyBlockHash,
    MicroBlockHash,
    BlockPofHash,
    BlockTxHash,
    BlockStateHash,
    Channel,
    ContractBytearray,
    ContractPubkey,
    ContractStoreKey,
    ContractStoreValue,
    Transaction,
    TxHash,
    OraclePubkey,
    OracleQuery,
    OracleQueryId,
    OracleResponse,
    AccountPubkey,
    Signature,
    Name,
    Commitment,
    PeerPubkey,
    State,
    Poi,
    StateTrees,
    CallStateTree,
    Bytearray,
}

impl KnownType {
    pub fn byte_size(self) -> Option<usize> {
        match self {
            KnownType::KeyBlockHash => Some(32),
            KnownType::MicroBlockHash => Some(32),
            KnownType::BlockPofHash => Some(32),
            KnownType::BlockTxHash => Some(32),
            KnownType::BlockStateHash => Some(32),
            KnownType::Channel => Some(32),
            KnownType::ContractPubkey => Some(32),
            KnownType::ContractBytearray => None,
            KnownType::ContractStoreKey => None,
            KnownType::ContractStoreValue => None,
            KnownType::Transaction => None,
            KnownType::TxHash => Some(32),
            KnownType::OraclePubkey => Some(32),
            KnownType::OracleQuery => None,
            KnownType::OracleQueryId => Some(32),
            KnownType::OracleResponse => None,
            KnownType::AccountPubkey => Some(32),
            KnownType::Signature => Some(64),
            KnownType::Name => None,
            KnownType::Commitment => Some(32),
            KnownType::PeerPubkey => Some(32),
            KnownType::State => Some(32),
            KnownType::Poi => None,
            KnownType::StateTrees => None,
            KnownType::CallStateTree => None,
            KnownType::Bytearray => None,
        }
    }

    pub fn check_size(self, s: usize) -> bool {
        match self.byte_size() {
            Some(n) => n == s,
            None => true,
        }
    }

    fn prefix(self) -> String {
        let s = match self {
            KnownType::KeyBlockHash => "kh",
            KnownType::MicroBlockHash => "mh",
            KnownType::BlockPofHash => "bf",
            KnownType::BlockTxHash => "bx",
            KnownType::BlockStateHash => "bs",
            KnownType::Channel => "ch",
            KnownType::ContractBytearray => "cb",
            KnownType::ContractPubkey => "ck",
            KnownType::ContractStoreKey => "cv",
            KnownType::ContractStoreValue => "ct",
            KnownType::Transaction => "tx",
            KnownType::TxHash => "th",
            KnownType::OraclePubkey => "ok",
            KnownType::OracleQuery => "ov",
            KnownType::OracleQueryId => "oq",
            KnownType::OracleResponse => "or",
            KnownType::AccountPubkey => "ak",
            KnownType::Signature => "sg",
            KnownType::Name => "cm",
            KnownType::Commitment => "pp",
            KnownType::PeerPubkey => "nm",
            KnownType::State => "st",
            KnownType::Poi => "pi",
            KnownType::StateTrees => "ss",
            KnownType::CallStateTree => "cs",
            KnownType::Bytearray => "ba",
        };
        String::from(s)
    }

    fn from_prefix(prefix: &str) -> Option<KnownType> {
        use KnownType::*;
        match prefix {
            "kh" => Some(KeyBlockHash),
            "mh" => Some(MicroBlockHash),
            "bf" => Some(BlockPofHash),
            "bx" => Some(BlockTxHash),
            "bs" => Some(BlockStateHash),
            "ch" => Some(Channel),
            "cb" => Some(ContractBytearray),
            "ck" => Some(ContractPubkey),
            "cv" => Some(ContractStoreKey),
            "ct" => Some(ContractStoreValue),
            "tx" => Some(Transaction),
            "th" => Some(TxHash),
            "ok" => Some(OraclePubkey),
            "ov" => Some(OracleQuery),
            "oq" => Some(OracleQueryId),
            "or" => Some(OracleResponse),
            "ak" => Some(AccountPubkey),
            "sg" => Some(Signature),
            "cm" => Some(Name),
            "pp" => Some(Commitment),
            "nm" => Some(PeerPubkey),
            "st" => Some(State),
            "pi" => Some(Poi),
            "ss" => Some(StateTrees),
            "cs" => Some(CallStateTree),
            "ba" => Some(Bytearray),
            _ => None,
        }
    }

    fn to_id_tag(self) -> Option<id::Tag> {
        use id::Tag;
        match self {
            KnownType::AccountPubkey => Some(Tag::Account),
            KnownType::Channel => Some(Tag::Channel),
            KnownType::Commitment => Some(Tag::Commitment),
            KnownType::ContractPubkey => Some(Tag::Contract),
            KnownType::Name => Some(Tag::Name),
            KnownType::OraclePubkey => Some(Tag::Oracle),
            _ => None,
        }
    }

    fn from_id_tag(tag: id::Tag) -> KnownType {
        use id::Tag;
        use KnownType::*;
        match tag {
            Tag::Account => AccountPubkey,
            Tag::Channel => Channel,
            Tag::Commitment => Commitment,
            Tag::Contract => ContractPubkey,
            Tag::Name => Name,
            Tag::Oracle => OraclePubkey,
        }
    }

    fn encoding(self) -> Encoding {
        use Encoding::*;
        match self {
            KnownType::KeyBlockHash => Base58,
            KnownType::MicroBlockHash => Base58,
            KnownType::BlockPofHash => Base58,
            KnownType::BlockTxHash => Base58,
            KnownType::BlockStateHash => Base58,
            KnownType::Channel => Base58,
            KnownType::ContractBytearray => Base58,
            KnownType::ContractPubkey => Base64,
            KnownType::ContractStoreKey => Base64,
            KnownType::ContractStoreValue => Base64,
            KnownType::Transaction => Base64,
            KnownType::TxHash => Base58,
            KnownType::OraclePubkey => Base58,
            KnownType::OracleQuery => Base64,
            KnownType::OracleQueryId => Base58,
            KnownType::OracleResponse => Base64,
            KnownType::AccountPubkey => Base58,
            KnownType::Signature => Base58,
            KnownType::Name => Base58,
            KnownType::Commitment => Base58,
            KnownType::PeerPubkey => Base58,
            KnownType::State => Base64,
            KnownType::Poi => Base64,
            KnownType::StateTrees => Base64,
            KnownType::CallStateTree => Base64,
            KnownType::Bytearray => Base64,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Encoding {
    Base58,
    Base64,
}

impl Encoding {
    fn make_check(self, data: &[u8]) -> Bytes {
        use sha256::digest;
        let d = digest(digest(data));
        d.as_bytes()[..4].to_vec()
    }

    fn add_check(self, data: &[u8]) -> Bytes {
        let c = self.make_check(data);
        vec![data, &c].concat()
    }

    fn encode(self, data: &[u8]) -> Bytes {
        match self {
            Encoding::Base58 => bs58::encode(data).into_vec(),
            Encoding::Base64 => base64::encode(data).as_bytes().to_vec(),
        }
    }

    fn encode_with_check(self, data: &[u8]) -> Bytes {
        let data_c = self.add_check(data);
        self.encode(&data_c)
    }

    fn decode(self, data: &[u8]) -> Option<Bytes> {
        match self {
            Encoding::Base58 => bs58::decode(data).into_vec().ok(),
            Encoding::Base64 => base64::decode(data).ok()
        }
    }
}

pub fn encode_data(t: KnownType, payload: Bytes) -> Bytes {
    let pfx = t.prefix();
    let enc = t.encoding().encode_with_check(&payload);
    pfx.bytes()
        .into_iter()
        .chain("_".bytes())
        .chain(enc)
        .collect()
}

pub fn encode_id(id: id::Id) -> Bytes {
    encode_data(KnownType::from_id_tag(id.tag), id.val.to_vec())
}

type Error = u32; // TODO normal error

pub fn decode(data: Bytes) -> Result<(KnownType, Bytes), Error> {
    let (pfx, payload) = split_prefix(&data)?;
    let tp = KnownType::from_prefix(&pfx).ok_or(2135 as Error)?;
    let decoded = decode_check(tp, payload)?;
    if tp.check_size(decoded.len()) {
        Ok((tp, decoded))
    } else {
        Err(2137)
    }
}

fn split_prefix(data: &[u8]) -> Result<(String, Bytes), Error> {
    let pfx = String::from_utf8(data[0..2].to_vec()).map_err(|_| 2134 as Error)?;
    let payload = data[3..].to_vec();
    Ok((pfx, payload))
}

fn decode_check(tp: KnownType, data: Bytes) -> Result<Bytes, Error> {
    let dec = tp.encoding().decode(&data).ok_or(2133 as Error)?;
    let body_size = dec.len() - 4;
    let body = &dec[0..body_size];
    let c = &dec[body_size..body_size + 4];
    assert_eq!(c, tp.encoding().make_check(body));
    Ok(body.to_vec())
}

pub fn decode_id(allowed_types: Vec<KnownType>, data: Bytes) -> Result<id::Id, Error> {
    let (tp, decoded) = decode(data)?;

    let val: [u8; 32] = decoded.try_into().map_err(|_| 2131 as Error)?;

    if allowed_types.contains(&tp) {
        match tp.to_id_tag() {
            Some(tag) => Ok(id::Id{tag: tag, val: val}),
            None => Err(2139)
        }
    } else {
        Err(2138)
    }
}

pub fn decode_blockhash(data: Bytes) -> Result<Bytes, Error> {
    let (tp, decoded) = decode(data)?;
    match tp {
        KnownType::KeyBlockHash => Ok(decoded),
        KnownType::MicroBlockHash => Ok(decoded),
        _ => Err(2136)
    }
}
