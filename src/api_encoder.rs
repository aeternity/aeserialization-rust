use crate::error::DecodingErr;
use crate::id;
use crate::Bytes;

/// Possible chain-object types.
#[derive(Debug, Copy, Clone, PartialEq)]
#[derive(rustler::NifTaggedEnum)]
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
    /// Payload size for a given type. Returns [None] is the size is not fixed.
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

    /// Validates payload size. Returns [true] when the size for the type matches or the type does
    /// not constrain the size. Returns [false] otherwise.
    pub fn check_size(self, s: usize) -> bool {
        match self.byte_size() {
            Some(n) => n == s,
            None => true,
        }
    }

    /// Returns a prefix describing the type. This prefix is prepended to the encoded payload and
    /// separated with a single '_' character.
    pub fn prefix(self) -> String {
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

    /// Parses the type from a prefix. See [to_prefix] for more details.
    pub fn from_prefix(prefix: &str) -> Option<KnownType> {
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

    /// Describes how payload is encoded.
    pub fn encoding(self) -> Encoding {
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

/// Supported types of encoding.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Encoding {
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

    fn encode(self, data: &[u8]) -> String {
        match self {
            Encoding::Base58 => bs58::encode(data).into_string(),
            Encoding::Base64 => {
                use base64::engine::general_purpose::STANDARD;
                use base64::Engine;
                STANDARD.encode(data)
            }
        }
    }

    fn encode_with_check(self, data: &[u8]) -> String {
        let data_c = self.add_check(data);
        self.encode(&data_c)
    }

    fn decode(self, data: &str) -> Option<Bytes> {
        match self {
            Encoding::Base58 => bs58::decode(data).into_vec().ok(),
            Encoding::Base64 => {
                use base64::Engine;
                use base64::engine::general_purpose::STANDARD;
                STANDARD.decode(data).ok()
            }
        }
    }
}

/// Encodes raw data accordingly to the type. Includes a checksum.
pub fn encode_data(t: KnownType, payload: &[u8]) -> String {
    let pfx = t.prefix();
    let enc = t.encoding().encode_with_check(payload);
    [&pfx, "_", &enc].concat()
}

/// Encodes an id. Includes a checksum.
pub fn encode_id(id: &id::Id) -> String {
    encode_data(KnownType::from_id_tag(id.tag), &id.val.bytes)
}

/// Decodes raw data according to the prefixed type.
pub fn decode(data: &str) -> Result<(KnownType, Bytes), DecodingErr> {
    let (pfx, payload) = split_prefix(data)?;
    let tp = KnownType::from_prefix(&pfx).ok_or(DecodingErr::InvalidPrefix)?;
    let decoded = decode_check(tp, &payload)?;

    if !tp.check_size(decoded.len()) {
        Err(DecodingErr::IncorrectSize)?;
    }

    Ok((tp, decoded))
}

fn split_prefix(data: &str) -> Result<(String, String), DecodingErr> {
    let (pfx, payload) = data.split_once('_').ok_or(DecodingErr::MissingPrefix)?;

    if pfx.len() != 2 {
        Err(DecodingErr::InvalidPrefix)?;
    }

    Ok((pfx.to_string(), payload.to_string()))
}

fn decode_check(tp: KnownType, data: &str) -> Result<Bytes, DecodingErr> {
    let dec = tp
        .encoding()
        .decode(data)
        .ok_or(DecodingErr::InvalidEncoding)?;
    let body_size = dec.len() - 4;
    let body = &dec[0..body_size];
    let c = &dec[body_size..body_size + 4];
    assert_eq!(c, tp.encoding().make_check(body));

    Ok(body.to_vec())
}

/// Decodes data as an id.
pub fn decode_id(allowed_types: &[KnownType], data: &str) -> Result<id::Id, DecodingErr> {
    let (tp, decoded) = decode(data)?;

    let val: [u8; 32] = decoded
        .try_into()
        .map_err(|_| DecodingErr::InvalidEncoding)?;

    if !allowed_types.contains(&tp) {
        Err(DecodingErr::InvalidPrefix)?;
    }

    let id = id::Id {
        tag: tp.to_id_tag().ok_or(DecodingErr::InvalidPrefix)?,
        val: id::EncodedId{bytes: val}
    };
    Ok(id)
}

/// Decodes a block hash. Requires an adequate prefix.
pub fn decode_blockhash(data: &str) -> Result<Bytes, DecodingErr> {
    let (tp, decoded) = decode(data)?;
    match tp {
        KnownType::KeyBlockHash => Ok(decoded),
        KnownType::MicroBlockHash => Ok(decoded),
        _ => Err(DecodingErr::InvalidPrefix),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    impl proptest::arbitrary::Arbitrary for KnownType {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(KnownType::KeyBlockHash),
                Just(KnownType::MicroBlockHash),
                Just(KnownType::BlockPofHash),
                Just(KnownType::BlockTxHash),
                Just(KnownType::BlockStateHash),
                Just(KnownType::Channel),
                Just(KnownType::ContractBytearray),
                Just(KnownType::ContractPubkey),
                Just(KnownType::ContractStoreKey),
                Just(KnownType::ContractStoreValue),
                Just(KnownType::Transaction),
                Just(KnownType::TxHash),
                Just(KnownType::OraclePubkey),
                Just(KnownType::OracleQuery),
                Just(KnownType::OracleQueryId),
                Just(KnownType::OracleResponse),
                Just(KnownType::AccountPubkey),
                Just(KnownType::Signature),
                Just(KnownType::Name),
                Just(KnownType::Commitment),
                Just(KnownType::PeerPubkey),
                Just(KnownType::State),
                Just(KnownType::Poi),
                Just(KnownType::StateTrees),
                Just(KnownType::CallStateTree),
                Just(KnownType::Bytearray),
            ].boxed()
        }
    }

    impl proptest::arbitrary::Arbitrary for Encoding {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(Encoding::Base58),
                Just(Encoding::Base64),
            ].boxed()
        }
    }

    fn valid_data() -> impl Strategy<Value = (KnownType, Bytes)> {
        any::<KnownType>().prop_flat_map(|tp| {
            let (min, max) = match tp.byte_size() {
                Some(s) => (s, s),
                None => (0, 256)
            };
            prop::collection::vec(any::<u8>(), min..=max).prop_map(move |data| (tp, data))
        })
    }


    prop_compose!{
        fn known_types_with
            (tp: KnownType, max_elems: usize)
            (vec_l in prop::collection::vec(any::<KnownType>(), 1..max_elems/2),
             vec_r in prop::collection::vec(any::<KnownType>(), 1..max_elems/2)
            )
             -> Vec<KnownType>
        {
            vec![&vec_l[..], &[tp], &vec_r[..]].concat()
        }
    }

    fn known_types_without
            (tp: KnownType, max_elems: usize)
            -> impl Strategy<Value = Vec<KnownType>>
    {
        prop::collection::vec(any::<KnownType>().prop_filter("Unwanted type", move |t| *t != tp), 1..max_elems)
    }

    proptest! {
        #[test]
        fn prefix_roundtrip(tp: KnownType) {
            let tp1 = KnownType::from_prefix(&tp.prefix());
            prop_assert_eq!(Some(tp), tp1);
        }

        #[test]
        fn encoding_and_prefix((tp, data) in valid_data()) {
            let pfx = tp.prefix();
            let enc = encode_data(tp, &data);
            prop_assert_eq!(enc.as_bytes()[2], b'_');
            let (pfx1, enc_data) = split_prefix(&enc).expect("Prefix split");
            prop_assert_eq!(pfx1, pfx);
            prop_assert_eq!(enc_data, &enc[3..]);
        }

        #[test]
        fn encoding_roundtrip((tp, data) in valid_data()) {
            let enc = encode_data(tp, &data);
            let (tp1, data1) = decode(&enc).expect("Decoding failed");
            prop_assert_eq!(tp, tp1);
            prop_assert_eq!(data, data1);
        }

        #[test]
        fn encoding_id_roundtrip(
            val: [u8; 32],
            (t, allowed_types) in
                any::<id::Tag>().prop_flat_map(|tag| (Just(tag), known_types_with(KnownType::from_id_tag(tag), 5))))
            {

                let id = id::Id{tag: t, val: id::EncodedId{bytes: val}};
            let enc = encode_id(&id);
            let dec = decode_id(&allowed_types, &enc).expect("Decoding id failed");
            prop_assert_eq!(id, dec);
        }

        #[test]
        fn encoding_id_roundtrip_fail(
            val: [u8; 32],
            (t, allowed_types) in
                any::<id::Tag>().prop_flat_map(|tag| (Just(tag), known_types_without(KnownType::from_id_tag(tag), 5))))
            {
                let id = id::Id{tag: t, val: id::EncodedId{bytes: val}};
            let enc = encode_id(&id);
            let dec = decode_id(&allowed_types, &enc);
            prop_assert_eq!(Err(DecodingErr::InvalidPrefix), dec);
        }

    }
}
