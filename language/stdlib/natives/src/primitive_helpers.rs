use crate::dispatch::{CostedReturnType, NativeReturnType, Result, StackAccessor};
use byteorder::{LittleEndian, WriteBytesExt};
use types::byte_array::ByteArray;

pub fn native_bytearray_concat<T: StackAccessor>(mut accessor: T) -> Result<CostedReturnType> {
    let arg2 = accessor.get_byte_array()?;
    let arg1 = accessor.get_byte_array()?;
    let mut return_val = arg1.as_bytes().to_vec();
    return_val.extend_from_slice(arg2.as_bytes());

    // TODO: Figure out the gas cost for concatenation.
    let native_cost = return_val.len() as u64;
    let native_return = NativeReturnType::ByteArray(ByteArray::new(return_val));
    Ok(CostedReturnType::new(native_cost, native_return))
}

pub fn native_address_to_bytes<T: StackAccessor>(mut accessor: T) -> Result<CostedReturnType> {
    let arg = accessor.get_address()?;
    let return_val = arg.to_vec();

    // TODO: Figure out the gas cost for conversion.
    let native_cost = return_val.len() as u64;
    let native_return = NativeReturnType::ByteArray(ByteArray::new(return_val));
    Ok(CostedReturnType::new(native_cost, native_return))
}

pub fn native_u64_to_bytes<T: StackAccessor>(mut accessor: T) -> Result<CostedReturnType> {
    let arg = accessor.get_u64()?;
    let mut return_val: Vec<u8> = vec![];
    return_val.write_u64::<LittleEndian>(arg)?;

    // TODO: Figure out the gas cost for conversion.
    let native_cost = return_val.len() as u64;
    let native_return = NativeReturnType::ByteArray(ByteArray::new(return_val));
    Ok(CostedReturnType::new(native_cost, native_return))
}
