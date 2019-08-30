use crate::{
    native_structs::{
        def::NativeStructTag, vector::NativeVector, NativeStructType, NativeStructValue,
    },
    value::{deserialize_value, MutVal},
};
use canonical_serialization::*;
use failure::prelude::*;
use types::vm_error::StatusCode;
use vm::errors::*;

impl CanonicalSerialize for NativeVector {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u64(self.0.len() as u64)?;
        for elem in self.0.iter() {
            elem.serialize(serializer)?
        }
        Ok(())
    }
}

pub(crate) fn deserialize_vector(
    deserializer: &mut SimpleDeserializer,
    ty: &NativeStructType,
) -> VMResult<NativeVector> {
    let vec_length = if let Ok(len) = deserializer.decode_u64() {
        len
    } else {
        return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
    };
    let mut val = vec![];
    if ty.type_actuals().len() != 1 {
        return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
    };
    let elem_type = &ty.type_actuals()[0];
    for _i in 0..vec_length {
        val.push(MutVal::new(deserialize_value(deserializer, elem_type)?));
    }
    Ok(NativeVector(val))
}

impl CanonicalSerialize for NativeStructValue {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            NativeStructValue::Vector(v) => {
                serializer.encode_struct(v)?;
            }
        }
        Ok(())
    }
}

pub(crate) fn deserialize_native(
    deserializer: &mut SimpleDeserializer,
    ty: &NativeStructType,
) -> VMResult<NativeStructValue> {
    match &ty.tag {
        NativeStructTag::Vector => Ok(NativeStructValue::Vector(deserialize_vector(
            deserializer,
            ty,
        )?)),
    }
}
