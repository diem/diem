// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::SignatureChecker;
use invalid_mutations::signature::{
    ApplySignatureDoubleRefContext, ApplySignatureFieldRefContext, DoubleRefMutation,
    FieldRefMutation,
};
use proptest::{collection::vec, prelude::*};
use types::vm_error::StatusCode;
use vm::file_format::CompiledModule;

proptest! {
    #[test]
    fn valid_signatures(module in CompiledModule::valid_strategy(20)) {
        let signature_checker = SignatureChecker::new(&module);
        prop_assert_eq!(signature_checker.verify(), vec![]);
    }

    #[test]
    fn double_refs(
        module in CompiledModule::valid_strategy(20),
        mutations in vec(DoubleRefMutation::strategy(), 0..40),
    ) {
        let mut module = module.into_inner();
        let mut expected_violations = {
            let context = ApplySignatureDoubleRefContext::new(&mut module, mutations);
            context.apply()
        };
        expected_violations.sort();
        let module = module.freeze().expect("should satisfy bounds checker");

        let signature_checker = SignatureChecker::new(&module);

        let actual_violations = signature_checker.verify();
        // Since some type signatures are field definition references as well, actual_violations
        // will also contain VMStaticViolation::InvalidFieldDefReference errors -- filter those
        // out.
        let mut actual_violations: Vec<_> = actual_violations
            .into_iter()
            .filter(|err| err.major_status != StatusCode::INVALID_FIELD_DEF_REFERENCE)
            .collect();
        actual_violations.sort();
        // The error messages are slightly different from the invalid mutations, so clean these out
        for violation in actual_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        for violation in expected_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        prop_assert_eq!(expected_violations, actual_violations);
    }

    #[test]
    fn field_def_references(
        module in CompiledModule::valid_strategy(20),
        mutations in vec(FieldRefMutation::strategy(), 0..40),
    ) {
        let mut module = module.into_inner();
        let mut expected_violations = {
            let context = ApplySignatureFieldRefContext::new(&mut module, mutations);
            context.apply()
        };
        expected_violations.sort();
        let module = module.freeze().expect("should satisfy bounds checker");

        let signature_checker = SignatureChecker::new(&module);

        let mut actual_violations = signature_checker.verify();
        // Note that this shouldn't cause any InvalidSignatureToken errors because there are no
        // double references involved. So no filtering is required here.
        actual_violations.sort();
        // The error messages are slightly different from the invalid mutations, so clean these out
        for violation in actual_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        for violation in expected_violations.iter_mut() {
            violation.set_message("".to_string())
        }
        prop_assert_eq!(expected_violations, actual_violations);
    }
}
