//! Definition of errors.

use std::error::Error;
use std::fmt;

/// A specialized Result type for Vibrato.
pub type Result<T, E = VibratoError> = std::result::Result<T, E>;

/// The error type for Vaporetto.
#[derive(Debug)]
pub enum VibratoError {
    /// The error variant for [`InvalidArgumentError`].
    InvalidArgument(InvalidArgumentError),

    /// The error variant for [`TryFromIntError`](std::num::TryFromIntError).
    TryFromInt(std::num::TryFromIntError),

    /// The error variant for [`ParseIntError`](std::num::ParseIntError).
    ParseInt(std::num::ParseIntError),

    /// The error variant for [`DecodeError`](bincode::error::DecodeError).
    BincodeDecode(bincode::error::DecodeError),

    /// The error variant for [`EncodeError`](bincode::error::EncodeError).
    BincodeEncode(bincode::error::EncodeError),

    /// The error variant for [`std::io::Error`].
    StdIo(std::io::Error),
}

impl VibratoError {
    pub(crate) fn invalid_argument<S>(arg: &'static str, msg: S) -> Self
    where
        S: Into<String>,
    {
        Self::InvalidArgument(InvalidArgumentError {
            arg,
            msg: msg.into(),
        })
    }
}

impl fmt::Display for VibratoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidArgument(e) => e.fmt(f),
            Self::TryFromInt(e) => e.fmt(f),
            Self::ParseInt(e) => e.fmt(f),
            Self::BincodeDecode(e) => e.fmt(f),
            Self::BincodeEncode(e) => e.fmt(f),
            Self::StdIo(e) => e.fmt(f),
        }
    }
}

impl Error for VibratoError {}

/// Error used when the argument is invalid.
#[derive(Debug)]
pub struct InvalidArgumentError {
    /// Name of the argument.
    pub(crate) arg: &'static str,

    /// Error message.
    pub(crate) msg: String,
}

impl fmt::Display for InvalidArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidArgumentError: {}: {}", self.arg, self.msg)
    }
}

impl Error for InvalidArgumentError {}

impl From<std::num::TryFromIntError> for VibratoError {
    fn from(error: std::num::TryFromIntError) -> Self {
        Self::TryFromInt(error)
    }
}

impl From<std::num::ParseIntError> for VibratoError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl From<bincode::error::DecodeError> for VibratoError {
    fn from(error: bincode::error::DecodeError) -> Self {
        Self::BincodeDecode(error)
    }
}

impl From<bincode::error::EncodeError> for VibratoError {
    fn from(error: bincode::error::EncodeError) -> Self {
        Self::BincodeEncode(error)
    }
}

impl From<std::io::Error> for VibratoError {
    fn from(error: std::io::Error) -> Self {
        Self::StdIo(error)
    }
}
