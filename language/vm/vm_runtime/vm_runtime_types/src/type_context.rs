use crate::loaded_data::{struct_def::StructDef, types::Type};
use types::vm_error::{StatusCode, VMStatus};

pub struct TypeContext(Vec<Type>);

impl TypeContext {
    pub fn new(ty: Vec<Type>) -> Self {
        Self(ty)
    }

    pub fn identity_mapping(num_type_args: u16) -> Self {
        Self((0..num_type_args).map(Type::TypeVariable).collect())
    }

    pub fn subst_type(&self, ty: &Type) -> Result<Type, VMStatus> {
        Ok(match ty {
            Type::TypeVariable(idx) => self.get_type(*idx)?,
            Type::Reference(ty) => Type::Reference(Box::new(self.subst_type(ty)?)),
            Type::MutableReference(ty) => Type::MutableReference(Box::new(self.subst_type(ty)?)),
            Type::Struct(s) => Type::Struct(self.subst_struct_def(s)?),
            id => id.clone(),
        })
    }

    pub fn subst_struct_def(&self, def: &StructDef) -> Result<StructDef, VMStatus> {
        Ok(StructDef::new(
            def.field_definitions()
                .iter()
                .map(|ty| self.subst_type(ty))
                .collect::<Result<_, _>>()?,
        ))
    }

    pub fn get_type(&self, idx: u16) -> Result<Type, VMStatus> {
        self.0
            .get(idx as usize)
            .cloned()
            .ok_or_else(|| VMStatus::new(StatusCode::INTERNAL_TYPE_ERROR))
    }
}
