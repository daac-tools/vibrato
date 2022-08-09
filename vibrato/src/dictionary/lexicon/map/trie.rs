use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};

use crate::errors::{Result, VibratoError};

pub(crate) struct Trie {
    da: crawdad::Trie,
}

impl Encode for Trie {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.da.serialize_to_vec(), encoder)?;
        Ok(())
    }
}

impl Decode for Trie {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let data: Vec<u8> = Decode::decode(decoder)?;
        let (da, _) = crawdad::Trie::deserialize_from_slice(&data);
        Ok(Self { da })
    }
}

impl Trie {
    pub fn from_records<K>(records: &[(K, u32)]) -> Result<Self>
    where
        K: AsRef<str>,
    {
        Ok(Self {
            da: crawdad::Trie::from_records(records.iter().map(|(k, v)| (k, *v)))
                .map_err(|e| VibratoError::invalid_argument("records", e.to_string()))?,
        })
    }

    #[inline(always)]
    pub fn common_prefix_iterator<'a>(
        &'a self,
        input: &'a [char],
    ) -> impl Iterator<Item = TrieMatch> + 'a {
        debug_assert!(input.len() <= 0xFFFF);
        self.da
            .common_prefix_search(input.iter().cloned())
            .map(move |(value, end_char)| {
                // Safety: input.len() is no more than 0xFFFF.
                TrieMatch::new(value, unsafe { u16::try_from(end_char).unwrap_unchecked() })
            })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) struct TrieMatch {
    pub value: u32,
    pub end_char: u16,
}

impl TrieMatch {
    #[inline(always)]
    pub const fn new(value: u32, end_char: u16) -> Self {
        Self { value, end_char }
    }
}
