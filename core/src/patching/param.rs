use serde::{Deserialize, Serialize};

pub trait ParamType {
    type InternalType;

    const SERIALIZED_NAME: &str;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Endiannness {
    Little,
    Big,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParameterBase<T: ParamType> {
    name: String,

    #[serde(rename = "type")]
    type_name: T::InternalType,

    endianness: Endiannness,

    default: Option<T>,
    min: Option<T>,
    max: Option<T>,

    value: T,
}

macro_rules! impl_param_type {
    ($t:ty) => {
        impl ParamType for $t {
            type InternalType = $t;
            const SERIALIZED_NAME: &str = stringify!($t);
        }
    };
}

impl_param_type!(f32);
impl_param_type!(f64);
impl_param_type!(u8);
impl_param_type!(i8);
impl_param_type!(u16);
impl_param_type!(i16);
impl_param_type!(u32);
impl_param_type!(i32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Parameter {
    F32(ParameterBase<f32>),
    F64(ParameterBase<f64>),
    U8(ParameterBase<u8>),
    I8(ParameterBase<i8>),
    U16(ParameterBase<u16>),
    I16(ParameterBase<i16>),
    U32(ParameterBase<u32>),
    I32(ParameterBase<i32>),
}
