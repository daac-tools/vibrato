//! Utilities for serializing/deserializing data.

use crate::utils::FromU32;

/// Trait indicating serializability.
///
/// If the type of output value of the automaton implements this trait, the automaton can be
/// serialized.
pub trait Serializable: Sized {
    /// A function called during serialization.
    ///
    /// # Arguments
    ///
    /// * `dst` - the destination to which the serialized data is written.
    fn serialize_to_vec(&self, dst: &mut Vec<u8>);

    /// A function called during deserialization. This function must return the pair of the struct
    /// and the rest slice.
    ///
    /// # Arguments
    ///
    /// * `src` - the source slice containing the serialized data.
    fn deserialize_from_slice(src: &[u8]) -> (Self, &[u8]);

    /// Returns the size of serialized data.
    fn serialized_bytes() -> usize;
}

macro_rules! define_serializable_primitive {
    ($type:ty, $size:expr) => {
        impl Serializable for $type {
            #[inline(always)]
            fn serialize_to_vec(&self, dst: &mut Vec<u8>) {
                dst.extend_from_slice(&self.to_le_bytes());
            }

            #[inline(always)]
            fn deserialize_from_slice(src: &[u8]) -> (Self, &[u8]) {
                let x = Self::from_le_bytes(src[..$size].try_into().unwrap());
                (x, &src[$size..])
            }

            #[inline(always)]
            fn serialized_bytes() -> usize {
                $size
            }
        }
    };
}

define_serializable_primitive!(u8, 1);
define_serializable_primitive!(u16, 2);
define_serializable_primitive!(u32, 4);
define_serializable_primitive!(u64, 8);
define_serializable_primitive!(u128, 16);
#[cfg(target_pointer_width = "32")]
define_serializable_primitive!(usize, 4);
#[cfg(target_pointer_width = "64")]
define_serializable_primitive!(usize, 8);

define_serializable_primitive!(i8, 1);
define_serializable_primitive!(i16, 2);
define_serializable_primitive!(i32, 4);
define_serializable_primitive!(i64, 8);
define_serializable_primitive!(i128, 16);
#[cfg(target_pointer_width = "32")]
define_serializable_primitive!(isize, 4);
#[cfg(target_pointer_width = "64")]
define_serializable_primitive!(isize, 8);

pub trait SerializableVec: Sized {
    fn serialize_to_vec(&self, dst: &mut Vec<u8>);

    fn deserialize_from_slice(src: &[u8]) -> (Self, &[u8]);

    fn serialized_bytes(&self) -> usize;
}

impl<S> SerializableVec for Vec<S>
where
    S: Serializable,
{
    #[inline(always)]
    fn serialize_to_vec(&self, dst: &mut Vec<u8>) {
        u32::try_from(self.len()).unwrap().serialize_to_vec(dst);
        self.iter().for_each(|x| x.serialize_to_vec(dst));
    }

    #[inline(always)]
    fn deserialize_from_slice(src: &[u8]) -> (Self, &[u8]) {
        let (len, mut src) = u32::deserialize_from_slice(src);
        let mut dst = Self::with_capacity(usize::from_u32(len));
        for _ in 0..len {
            let (x, rest) = S::deserialize_from_slice(src);
            dst.push(x);
            src = rest;
        }
        (dst, src)
    }

    fn serialized_bytes(&self) -> usize {
        u32::serialized_bytes() + S::serialized_bytes() * self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32() {
        let x = 0x01234567u32;
        let mut data = vec![];
        x.serialize_to_vec(&mut data);
        assert_eq!(vec![0x67, 0x45, 0x23, 0x01], data);
        assert_eq!(4, u32::serialized_bytes());
        data.push(42);
        let (y, rest) = u32::deserialize_from_slice(&data);
        assert_eq!(&[42], rest);
        assert_eq!(x, y);
    }

    #[test]
    fn test_vec_u32() {
        let x = vec![0x01234567u32, 0x89abcdefu32, 0x02468aceu32];
        let mut data = vec![];
        x.serialize_to_vec(&mut data);
        assert_eq!(
            vec![
                0x03, 0x00, 0x00, 0x00, // len
                0x67, 0x45, 0x23, 0x01, // item 1
                0xef, 0xcd, 0xab, 0x89, // item 2
                0xce, 0x8a, 0x46, 0x02, // item 3
            ],
            data
        );
        assert_eq!(16, x.serialized_bytes());
        data.push(42);
        let (y, rest) = Vec::<u32>::deserialize_from_slice(&data);
        assert_eq!(&[42], rest);
        assert_eq!(x, y);
    }
}
