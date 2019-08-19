use vm::{
    errors::VMInvariantViolation,
    file_format::{SignatureToken, TypeParameterIndex},
};

#[cfg(test)]
#[path = "unit_tests/substitution_map_tests.rs"]
mod substitution_map_tests;

pub struct SubstitutionMap(Vec<SignatureToken>);

impl SubstitutionMap {
    pub fn new() -> Self {
        SubstitutionMap(vec![])
    }

    fn new_signature(&self, tok: SignatureToken) -> Result<SignatureToken, VMInvariantViolation> {
        Ok(match tok {
            SignatureToken::TypeParameter(idx) => self
                .0
                .get(idx as usize)
                .ok_or(VMInvariantViolation::InternalTypeError)?
                .clone(),
            SignatureToken::Struct(idx, type_actuals) => SignatureToken::Struct(
                idx,
                type_actuals
                    .into_iter()
                    .map(|tok| self.new_signature(tok))
                    .collect::<Result<Vec<SignatureToken>, VMInvariantViolation>>()?,
            ),
            SignatureToken::MutableReference(_) | SignatureToken::Reference(_) => {
                return Err(VMInvariantViolation::InternalTypeError)
            }
            id => id,
        })
    }

    pub fn subst(
        &self,
        toks: Vec<SignatureToken>,
    ) -> Result<SubstitutionMap, VMInvariantViolation> {
        Ok(SubstitutionMap(
            toks.into_iter()
                .map(|tok| self.new_signature(tok))
                .collect::<Result<Vec<SignatureToken>, VMInvariantViolation>>()?,
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
            .clone())
    }
}
