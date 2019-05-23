#[macro_export]
macro_rules! auto_serialize {
    ( $name:ident , {
        $( $field:ident : $ty:ident ),* ,
    } ) => {
        impl $name {
            pub fn deserialize<R: std::io::Read>(input: &mut R) -> std::io::Result<$name> {
                use $crate::types::*;
                $(
                    let $field = $ty::deserialize(input)?;
                )*
                Ok($name { $( $field ),* })
            }
            pub fn serialize<W: std::io::Write>(output: &mut W, val: &$name) -> std::io::Result<()> {
                use $crate::types::*;
                $(
                    $ty::serialize(output, val.$field)?;
                )*
                Ok(())
            }
        }
    }
}

macro_rules! builtin_num_type {
    ($name:ident, $ty:ident, $from_name:ident, $to_name:ident) => {
        impl $name {
            pub fn deserialize<R: std::io::Read>(input: &mut R) -> std::io::Result<$ty> {
                let mut bytes = [0; std::mem::size_of::<$ty>()];
                input.read_exact(&mut bytes)?;
                Ok($ty::$from_name(bytes))
            }
            pub fn serialize<W: std::io::Write>(output: &mut W, val: $ty) -> std::io::Result<()> {
                output.write_all(&val.$to_name()).map(|_| ())
            }
        }
    };
    ($name:ident, $ty:ident, le) => {
        builtin_num_type!($name, $ty, from_le_bytes, to_le_bytes);
    };
    ($name:ident, $ty:ident, be) => {
        builtin_num_type!($name, $ty, from_be_bytes, to_be_bytes);
    };
}

pub mod types {
    pub struct U128BE;
    builtin_num_type!(U128BE, u128, be);
    pub struct U128LE;
    builtin_num_type!(U128LE, u128, le);
    pub struct I128BE;
    builtin_num_type!(I128BE, i128, be);
    pub struct I128LE;
    builtin_num_type!(I128LE, i128, le);

    pub struct U64BE;
    builtin_num_type!(U64BE, u64, be);
    pub struct U64LE;
    builtin_num_type!(U64LE, u64, le);
    pub struct I64BE;
    builtin_num_type!(I64BE, i64, be);
    pub struct I64LE;
    builtin_num_type!(I64LE, i64, le);

    pub struct U32BE;
    builtin_num_type!(U32BE, u32, be);
    pub struct U32LE;
    builtin_num_type!(U32LE, u32, le);
    pub struct I32BE;
    builtin_num_type!(I32BE, i32, be);
    pub struct I32LE;
    builtin_num_type!(I32LE, i32, le);

    pub struct U16BE;
    builtin_num_type!(U16BE, u16, be);
    pub struct U16LE;
    builtin_num_type!(U16LE, u16, le);
    pub struct I16BE;
    builtin_num_type!(I16BE, i16, be);
    pub struct I16LE;
    builtin_num_type!(I16LE, i16, le);

    pub struct U8;
    builtin_num_type!(U8, u8, le);
    pub struct I8;
    builtin_num_type!(I8, i8, le);

    pub struct F64BE;
    impl F64BE {
        pub fn deserialize<R: std::io::Read>(input: &mut R) -> std::io::Result<f64> {
            U64BE::deserialize(input).map(|val| unsafe { *(&val as *const u64 as *const f64) })
        }
        pub fn serialize<W: std::io::Write>(output: &mut W, val: f64) -> std::io::Result<()> {
            U64BE::serialize(output, unsafe { *(&val as *const f64 as *const u64) })
        }
    }
    pub struct F64LE;
    impl F64LE {
        pub fn deserialize<R: std::io::Read>(input: &mut R) -> std::io::Result<f64> {
            U64LE::deserialize(input).map(|val| unsafe { *(&val as *const u64 as *const f64) })
        }
        pub fn serialize<W: std::io::Write>(output: &mut W, val: f64) -> std::io::Result<()> {
            U64LE::serialize(output, unsafe { *(&val as *const f64 as *const u64) })
        }
    }

    pub struct F32BE;
    impl F32BE {
        pub fn deserialize<R: std::io::Read>(input: &mut R) -> std::io::Result<f32> {
            U32BE::deserialize(input).map(|val| unsafe { *(&val as *const u32 as *const f32) })
        }
        pub fn serialize<W: std::io::Write>(output: &mut W, val: f32) -> std::io::Result<()> {
            U32BE::serialize(output, unsafe { *(&val as *const f32 as *const u32) })
        }
    }
    pub struct F32LE;
    impl F32LE {
        pub fn deserialize<R: std::io::Read>(input: &mut R) -> std::io::Result<f32> {
            U32LE::deserialize(input).map(|val| unsafe { *(&val as *const u32 as *const f32) })
        }
        pub fn serialize<W: std::io::Write>(output: &mut W, val: f32) -> std::io::Result<()> {
            U32LE::serialize(output, unsafe { *(&val as *const f32 as *const u32) })
        }
    }

    pub struct Bool;
    impl Bool {
        pub fn deserialize<R: std::io::Read>(input: &mut R) -> std::io::Result<bool> {
            U8::deserialize(input).map(|n| n != 0)
        }
        pub fn serialize<W: std::io::Write>(output: &mut W, val: bool) -> std::io::Result<()> {
            U8::serialize(output, if val { 1 } else { 0 })
        }
    }
}
