#[macro_export]
macro_rules! infallible_try_into {
    ($from:ident, $to:ty) => {
        impl std::convert::TryFrom<$from> for $to {
            type Error = std::convert::Infallible;
            fn try_from(n: $from) -> std::result::Result<Self, Self::Error> {
                n.0.try_into()
            }
        }
    };
}

#[macro_export]
macro_rules! fallible_try_into {
    ($from:ident, $to:ty) => {
        impl std::convert::TryFrom<$from> for $to {
            type Error = std::num::TryFromIntError;
            fn try_from(n: $from) -> std::result::Result<Self, Self::Error> {
                n.0.try_into()
            }
        }
    };
}

#[macro_export]
macro_rules! fallible_try_from {
    ($to:ty, $from:ident) => {
        impl std::convert::TryFrom<$from> for $to {
            type Error = std::num::TryFromIntError;
            fn try_from(n: $from) -> std::result::Result<Self, Self::Error> {
                n.try_into().map(Self)
            }
        }
    };
}
