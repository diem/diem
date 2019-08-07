use vm::{
    errors::VMInvariantViolation,
    file_format::{SignatureToken, StructHandleIndex, TypeParameterIndex},
};

#[cfg(test)]
#[path = "unit_tests/substitution_map_tests.rs"]
mod substitution_map_tests;

enum SignatureTokenRef<'a> {
    /// Boolean, `true` or `false`.
    Bool,
    /// Unsigned integers, 64 bits length.
    U64,
    /// Strings, immutable, utf8 representation.
    String,
    /// ByteArray, variable size, immutable byte array.
    ByteArray,
    /// Address, a 32 bytes immutable type.
    Address,
    /// MOVE user type, resource or unrestricted
    Struct(StructHandleIndex, Vec<SignatureTokenRef<'a>>),
    /// Type parameter.
    TypeParameter(&'a SignatureTokenRef<'a>),
}

pub struct SubstitutionMap<'a>(Vec<SignatureTokenRef<'a>>);

impl<'a> SignatureTokenRef<'a> {}

impl<'a> SignatureTokenRef<'a> {
    pub fn materialize(&self) -> SignatureToken {
        match self {
            SignatureTokenRef::Bool => SignatureToken::Bool,
            SignatureTokenRef::U64 => SignatureToken::U64,
            SignatureTokenRef::String => SignatureToken::String,
            SignatureTokenRef::ByteArray => SignatureToken::ByteArray,
            SignatureTokenRef::Address => SignatureToken::Address,
            SignatureTokenRef::Struct(idx, tok_vec) => {
                SignatureToken::Struct(*idx, tok_vec.iter().map(|tok| tok.materialize()).collect())
            }
            SignatureTokenRef::TypeParameter(ty) => ty.materialize(),
        }
    }
}

impl<'a> SubstitutionMap<'a> {
    pub fn new() -> Self {
        SubstitutionMap(vec![])
    }

    fn new_signature(
        &self,
        tok: SignatureToken,
    ) -> Result<SignatureTokenRef, VMInvariantViolation> {
        Ok(match tok {
            SignatureToken::Bool => SignatureTokenRef::Bool,
            SignatureToken::U64 => SignatureTokenRef::U64,
            SignatureToken::String => SignatureTokenRef::String,
            SignatureToken::ByteArray => SignatureTokenRef::ByteArray,
            SignatureToken::Address => SignatureTokenRef::Address,
            SignatureToken::Struct(idx, tok_vec) => SignatureTokenRef::Struct(
                idx,
                tok_vec
                    .into_iter()
                    .map(|tok| self.new_signature(tok))
                    .collect::<Result<Vec<SignatureTokenRef>, VMInvariantViolation>>()?,
            ),
            SignatureToken::Reference(_) | SignatureToken::MutableReference(_) => {
                return Err(VMInvariantViolation::InternalTypeError)
            }
            SignatureToken::TypeParameter(idx) => SignatureTokenRef::TypeParameter(
                self.0
                    .get(idx as usize)
                    .ok_or(VMInvariantViolation::InternalTypeError)?,
            ),
        })
    }

    pub fn subst(
        &self,
        toks: Vec<SignatureToken>,
    ) -> Result<SubstitutionMap, VMInvariantViolation> {
        Ok(SubstitutionMap(
            toks.into_iter()
                .map(|tok| self.new_signature(tok))
                .collect::<Result<Vec<SignatureTokenRef>, VMInvariantViolation>>()?,
        ))
    }

    pub fn materialize(
        &self,
        idx: TypeParameterIndex,
    ) -> Result<SignatureToken, VMInvariantViolation> {
        Ok(self
            .0
            .get(idx as usize)
            .ok_or(VMInvariantViolation::InternalTypeError)?
            .materialize())
    }
}
