use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_structs::vector::NativeVector,
};
use failure::prelude::*;
use libra_canonical_serialization::*;
use libra_vm::gas_schedule::{AbstractMemorySize, GasCarrier};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum NativeStructTag {
    Vector = 0,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct NativeStructType {
    pub tag: NativeStructTag,
    type_actuals: Vec<Type>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NativeStructValue {
    Vector(NativeVector),
}

impl NativeStructType {
    pub fn new(tag: NativeStructTag, type_actuals: Vec<Type>) -> Self {
        Self { tag, type_actuals }
    }
    pub fn new_vec(ty: Type) -> Self {
        Self {
            tag: NativeStructTag::Vector,
            type_actuals: vec![ty],
        }
    }

    pub fn type_actuals(&self) -> &[Type] {
        &self.type_actuals
    }
}

impl NativeStructValue {
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            NativeStructValue::Vector(v) => v.size(),
        }
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub(crate) fn to_struct_def_FOR_TESTING(&self) -> StructDef {
        match self {
            NativeStructValue::Vector(v) => StructDef::Native(NativeStructType::new_vec(
                v.get(0)
                    .map(|v| v.to_type_FOR_TESTING())
                    .unwrap_or(Type::Bool),
            )),
        }
    }
}

impl CanonicalSerialize for NativeStructTag {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            NativeStructTag::Vector => serializer.encode_u8(0x0)?,
        };
        Ok(())
    }
}

impl CanonicalDeserialize for NativeStructTag {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        Ok(match deserializer.decode_u8()? {
            0x0 => NativeStructTag::Vector,
            other => bail!(
                "Error while deserializing native type: found unexpected tag {:#x}",
                other
            ),
        })
    }
}

impl CanonicalSerialize for NativeStructType {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_struct(&self.tag)?;
        serializer.encode_vec(&self.type_actuals)?;
        Ok(())
    }
}

impl CanonicalDeserialize for NativeStructType {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        Ok(NativeStructType {
            tag: deserializer.decode_struct()?,
            type_actuals: deserializer.decode_vec()?,
        })
    }
}
