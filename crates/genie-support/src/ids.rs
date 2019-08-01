use crate::{fallible_try_from, fallible_try_into, infallible_try_into};
use std::convert::TryInto;

/// An ID identifying a unit type.
#[derive(Debug, Hash, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnitTypeID(u16);

impl From<u16> for UnitTypeID {
    fn from(n: u16) -> Self {
        UnitTypeID(n)
    }
}

impl From<UnitTypeID> for u16 {
    fn from(n: UnitTypeID) -> Self {
        n.0
    }
}

impl From<UnitTypeID> for i32 {
    fn from(n: UnitTypeID) -> Self {
        n.0.into()
    }
}

impl From<UnitTypeID> for u32 {
    fn from(n: UnitTypeID) -> Self {
        n.0.into()
    }
}

impl From<UnitTypeID> for usize {
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
    fn from(n: u16) -> Self {
        StringID(n.into())
    }
}

impl From<u32> for StringID {
    fn from(n: u32) -> Self {
        StringID(n)
    }
}

fallible_try_into!(StringID, u16);
fallible_try_into!(StringID, i16);
infallible_try_into!(StringID, u32);
fallible_try_into!(StringID, i32);
fallible_try_from!(StringID, i32);
