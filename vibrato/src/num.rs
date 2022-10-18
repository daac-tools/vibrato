use bincode::{
    de::Decoder,
    enc::Encoder,
    error::{AllowedEnumVariants, DecodeError, EncodeError},
    Decode, Encode,
};

/// Represents an integer from 0 to 2^31 - 1.
///
/// This type guarantees that the sign bit of a 32-bit integer is always zero.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct U31(u32);

impl U31 {
    pub const MAX: Self = Self(0x7fff_ffff);

    #[inline(always)]
    pub const fn new(x: u32) -> Option<Self> {
        if x <= Self::MAX.get() {
            Some(Self(x))
        } else {
            None
        }
    }

    #[inline(always)]
    pub const fn get(self) -> u32 {
        self.0
    }
}

const U31_VALID_RANGE: AllowedEnumVariants = AllowedEnumVariants::Range {
    min: 0,
    max: U31::MAX.get(),
};

impl Decode for U31 {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let x = Decode::decode(decoder)?;
        if let Some(x) = Self::new(x) {
            Ok(x)
        } else {
            Err(DecodeError::UnexpectedVariant {
                type_name: "U31",
                allowed: &U31_VALID_RANGE,
                found: x,
            })
        }
    }
}

bincode::impl_borrow_decode!(U31);

impl Encode for U31 {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.0, encoder)?;
        Ok(())
    }
}
