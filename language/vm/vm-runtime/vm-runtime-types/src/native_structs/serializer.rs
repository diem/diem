use crate::{
    native_structs::{
        def::NativeStructTag, vector::NativeVector, NativeStructType, NativeStructValue,
    },
    value::{deserialize_value, MutVal},
};
use libra_types::vm_error::StatusCode;
use serde::Deserialize;
use vm::errors::*;

pub(crate) fn deserialize_vector(
    deserializer: &mut lcs::Deserializer<'_>,
    ty: &NativeStructType,
) -> VMResult<NativeVector> {
    //TODO investigate if we can use `deserialize_seq` instead of reading the length explicitly
    let len = u32::deserialize(&mut *deserializer)
        .map_err(|_| vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR))?
        as usize;
    let mut val = Vec::with_capacity(len);
    if ty.type_actuals().len() != 1 {
        return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
    };
    let elem_type = &ty.type_actuals()[0];
    for _i in 0..len {
        val.push(MutVal::new(deserialize_value(deserializer, elem_type)?));
    }
    Ok(NativeVector(val))
}

pub(crate) fn deserialize_native(
    deserializer: &mut lcs::Deserializer<'_>,
    ty: &NativeStructType,
) -> VMResult<NativeStructValue> {
    match &ty.tag {
        NativeStructTag::Vector => Ok(NativeStructValue::Vector(deserialize_vector(
            deserializer,
            ty,
        )?)),
    }
}
