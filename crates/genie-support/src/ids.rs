use crate::{fallible_try_from, fallible_try_into, infallible_try_into};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    num::TryFromIntError,
};

/// An ID identifying a unit type.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnitTypeID(u16);

impl From<u16> for UnitTypeID {
    #[inline]
    fn from(n: u16) -> Self {
        UnitTypeID(n)
    }
}

impl From<UnitTypeID> for u16 {
    #[inline]
    fn from(n: UnitTypeID) -> Self {
        n.0
    }
}

impl From<UnitTypeID> for i32 {
    #[inline]
    fn from(n: UnitTypeID) -> Self {
        n.0.into()
    }
}

impl From<UnitTypeID> for u32 {
    #[inline]
    fn from(n: UnitTypeID) -> Self {
        n.0.into()
    }
}

impl From<UnitTypeID> for usize {
    #[inline]
    fn from(n: UnitTypeID) -> Self {
        n.0.into()
    }
}

fallible_try_into!(UnitTypeID, i16);
fallible_try_from!(UnitTypeID, i32);
fallible_try_from!(UnitTypeID, u32);

/// An ID identifying a string resource.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, Eq)]
pub struct StringID(u32);

impl From<u16> for StringID {
    #[inline]
    fn from(n: u16) -> Self {
        StringID(n.into())
    }
}

impl From<u32> for StringID {
    #[inline]
    fn from(n: u32) -> Self {
        StringID(n)
    }
}

fallible_try_into!(StringID, u16);
fallible_try_into!(StringID, i16);
infallible_try_into!(StringID, u32);
fallible_try_into!(StringID, i32);
fallible_try_from!(StringID, i32);

/// A key in a language file.
///
/// A key may be either a nonnegative integer or an arbitrary string.
///
/// The original game supports only nonnegative integers.
/// The HD Edition allows for integers as well as Strings to serve as keys in a
/// key value file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringKey {
    /// An integer string key.
    Num(u32),

    /// A named string key.
    /// The string must not represent a `u32` value (such keys must be `Num`).
    Name(String),
}

impl StringKey {
    /// Returns `true` if and only if this `StringKey` is a number.
    ///
    /// # Examples
    ///
    /// ```
    /// use genie_support::StringKey;
    /// assert!(StringKey::from(0).is_numeric());
    /// assert!(!StringKey::from("").is_numeric());
    /// ```
    #[inline]
    pub fn is_numeric(&self) -> bool {
        match self {
            Self::Num(_) => true,
            Self::Name(_) => false,
        }
    }

    /// Returns `true` if and only if this `StringKey` is a string name.
    ///
    /// # Examples
    ///
    /// ```
    /// use genie_support::StringKey;
    /// assert!(!StringKey::from(0).is_named());
    /// assert!(StringKey::from("").is_named());
    /// ```
    #[inline]
    pub fn is_named(&self) -> bool {
        match self {
            Self::Num(_) => false,
            Self::Name(_) => true,
        }
    }
}

impl fmt::Display for StringKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(n) => write!(f, "{}", n),
            Self::Name(s) => write!(f, "{}", s),
        }
    }
}

impl From<u32> for StringKey {
    #[inline]
    fn from(n: u32) -> Self {
        Self::Num(n)
    }
}

impl TryFrom<i32> for StringKey {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(n: i32) -> Result<Self, Self::Error> {
        u32::try_from(n).map(Self::Num)
    }
}

impl From<&str> for StringKey {
    #[inline]
    fn from(s: &str) -> Self {
        if let Ok(n) = s.parse() {
            Self::Num(n)
        } else {
            Self::Name(String::from(s))
        }
    }
}

impl From<String> for StringKey {
    #[inline]
    fn from(s: String) -> Self {
        Self::from(s.as_ref())
    }
}

/// Error that may occur when converting a StringKey to some other Rust value, such as an integer
/// or a string.
///
/// When converting to an integer, this means that the StringKey is a named key, or it has a
/// numeric value that is out of range for the target type.
///
/// When converting to a string, this does not happen, as numeric keys will be converted to
/// strings.
#[derive(Debug, Clone)]
pub struct TryFromStringKeyError;

impl fmt::Display for TryFromStringKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not convert StringKey to the wanted integer size")
    }
}

impl std::error::Error for TryFromStringKeyError {}

// Implement TryFrom<&StringKey> conversions for a bunch of stuff
macro_rules! try_from_string_key {
    ($type:ty) => {
        impl TryFrom<&StringKey> for $type {
            type Error = TryFromStringKeyError;
            #[inline]
            fn try_from(key: &StringKey) -> Result<Self, Self::Error> {
                match key {
                    StringKey::Num(n) => (*n).try_into().map_err(|_| TryFromStringKeyError),
                    _ => Err(TryFromStringKeyError),
                }
            }
        }
    };
}

try_from_string_key!(u32);
try_from_string_key!(i32);
try_from_string_key!(u16);
try_from_string_key!(i16);

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests converting from an int to a string key.
    #[test]
    fn string_key_from_int() {
        if let StringKey::Num(n) = StringKey::from(0) {
            assert_eq!(0, n);
        } else {
            panic!();
        }
    }

    /// Tests converting from a string representing an int to a string key.
    #[test]
    fn string_key_from_str_to_int() {
        let s = "57329";
        if let StringKey::Num(n) = StringKey::from(s) {
            assert_eq!(57329, n);
        } else {
            panic!();
        }
    }

    /// Tests converting from a string not representing an int to a string key.
    #[test]
    fn string_key_from_str_to_str() {
        let s = "grassDaut";
        if let StringKey::Name(n) = StringKey::from(s) {
            assert_eq!(s, n);
        } else {
            panic!();
        }
    }
}
