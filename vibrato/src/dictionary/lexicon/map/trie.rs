use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};

use crate::errors::{Result, VibratoError};

pub struct Trie {
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
        self.da
            .common_prefix_search(input.iter().cloned())
            // .map(move |(value, end_char)| TrieMatch::new(value, u32::try_from(end_char).unwrap()))
            .map(move |(value, end_char)| TrieMatch::new(value, end_char as u16))
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TrieMatch {
    pub value: u32,
    pub end_char: u16,
}

impl TrieMatch {
    #[inline(always)]
    pub const fn new(value: u32, end_char: u16) -> Self {
        Self { value, end_char }
    }
}
