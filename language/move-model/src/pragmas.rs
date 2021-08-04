// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides pragmas and properties of the specification language.

use crate::{ast::ConditionKind, builder::module_builder::SpecBlockContext};

/// Pragma indicating whether verification should be performed for a function.
pub const VERIFY_PRAGMA: &str = "verify";

/// Pragma defining a timeout.
pub const TIMEOUT_PRAGMA: &str = "timeout";

/// Pragma defining a random seed.
pub const SEED_PRAGMA: &str = "seed";

/// Pragma indicating an estimate how long verification takes. Verification
/// is skipped if the timeout is smaller than this.
pub const VERIFY_DURATION_ESTIMATE_PRAGMA: &str = "verify_duration_estimate";

/// Pragma indicating whether implementation of function should be ignored and
/// instead treated to be like a native function.
pub const INTRINSIC_PRAGMA: &str = "intrinsic";

/// Pragma indicating whether implementation of function should be ignored and
/// instead interpreted by its pre and post conditions only.
pub const OPAQUE_PRAGMA: &str = "opaque";

/// Pragma indicating whether emits specification should be considered partial.
pub const EMITS_IS_PARTIAL_PRAGMA: &str = "emits_is_partial";

/// Pragma indicating whether no emits specification should mean that no events are to be emitted.
pub const EMITS_IS_STRICT_PRAGMA: &str = "emits_is_strict";

/// Pragma indicating whether aborts_if specification should be considered partial.
pub const ABORTS_IF_IS_PARTIAL_PRAGMA: &str = "aborts_if_is_partial";

/// Pragma indicating whether no explicit aborts_if specification should be treated
/// like `aborts_if` false.
pub const ABORTS_IF_IS_STRICT_PRAGMA: &str = "aborts_if_is_strict";

/// Pragma indicating that requires are also enforced if the aborts condition is true.
pub const REQUIRES_IF_ABORTS_PRAGMA: &str = "requires_if_aborts";

/// Pragma indicating that the function will run smoke tests
pub const ALWAYS_ABORTS_TEST_PRAGMA: &str = "always_aborts_test";

/// Pragma indicating that adding u64 or u128 values should not be checked
/// for overflow.
pub const ADDITION_OVERFLOW_UNCHECKED_PRAGMA: &str = "addition_overflow_unchecked";

/// Pragma indicating that aborts from this function shall be ignored.
pub const ASSUME_NO_ABORT_FROM_HERE_PRAGMA: &str = "assume_no_abort_from_here";

/// Pragma which indicates that the function's abort and ensure conditions shall be exported
/// to the verification context even if the implementation of the function is inlined.
pub const EXPORT_ENSURES_PRAGMA: &str = "export_ensures";

/// Pragma indicating that the function can only be called from certain caller.
/// Unlike other pragmas, this pragma expects a function name like `0x1::M::f` instead
/// of a boolean or a number.
pub const FRIEND_PRAGMA: &str = "friend";

/// Pragma indicating that invariants are not to be checked between entry and exit
/// to this function
pub const DISABLE_INVARIANTS_IN_BODY_PRAGMA: &str = "disable_invariants_in_body";

/// Pragma indicating that invariants are not to be checked between entry and exit
/// to this function
pub const DELEGATE_INVARIANTS_TO_CALLER_PRAGMA: &str = "delegate_invariants_to_caller";

/// Checks whether a pragma is valid in a specific spec block.
pub fn is_pragma_valid_for_block(target: &SpecBlockContext<'_>, pragma: &str) -> bool {
    use crate::builder::module_builder::SpecBlockContext::*;
    match target {
        Module => matches!(
            pragma,
            VERIFY_PRAGMA
                | EMITS_IS_STRICT_PRAGMA
                | EMITS_IS_PARTIAL_PRAGMA
                | ABORTS_IF_IS_STRICT_PRAGMA
                | ABORTS_IF_IS_PARTIAL_PRAGMA
                | INTRINSIC_PRAGMA
        ),
        Function(..) => matches!(
            pragma,
            VERIFY_PRAGMA
                | TIMEOUT_PRAGMA
                | SEED_PRAGMA
                | VERIFY_DURATION_ESTIMATE_PRAGMA
                | INTRINSIC_PRAGMA
                | OPAQUE_PRAGMA
                | EMITS_IS_STRICT_PRAGMA
                | EMITS_IS_PARTIAL_PRAGMA
                | ABORTS_IF_IS_PARTIAL_PRAGMA
                | ABORTS_IF_IS_STRICT_PRAGMA
                | REQUIRES_IF_ABORTS_PRAGMA
                | ALWAYS_ABORTS_TEST_PRAGMA
                | ADDITION_OVERFLOW_UNCHECKED_PRAGMA
                | ASSUME_NO_ABORT_FROM_HERE_PRAGMA
                | EXPORT_ENSURES_PRAGMA
                | FRIEND_PRAGMA
                | DISABLE_INVARIANTS_IN_BODY_PRAGMA
                | DELEGATE_INVARIANTS_TO_CALLER_PRAGMA
        ),
        _ => false,
    }
}

/// Internal property attached to conditions if they are injected via an apply or a module
/// invariant.
pub const CONDITION_INJECTED_PROP: &str = "$injected";

/// Property which can be attached to conditions to make them exported into the VC context
/// even if they are injected.
pub const CONDITION_EXPORT_PROP: &str = "export";

/// Property which can be attached to a module invariant to make it global.
pub const CONDITION_GLOBAL_PROP: &str = "global";

/// Property which can be attached to a global invariant to mark it as not to be used as
/// an assumption in other verification steps. This can be used for invariants which are
/// nonoperational constraints on system behavior, i.e. the systems "works" whether the
/// invariant holds or not. Invariant marked as such are not assumed when
/// memory is accessed, but only in the pre-state of a memory update.
pub const CONDITION_ISOLATED_PROP: &str = "isolated";

/// Abstract property which can be used together with an opaque specification. An abstract
/// property is not verified against the implementation, but will be used for the
/// function's behavior in the application context. This allows to "override" the specification
/// with a more abstract version. In general we would need to prove the abstraction is
/// subsumed by the implementation, but this is currently not done.
pub const CONDITION_ABSTRACT_PROP: &str = "abstract";

/// Opposite to the abstract property.
pub const CONDITION_CONCRETE_PROP: &str = "concrete";

/// Property which indicates that an aborts_if should be assumed.
/// For callers of a function with such an aborts_if, the negation of the condition becomes
/// an assumption.
pub const CONDITION_ABORT_ASSUME_PROP: &str = "assume";

/// Property which indicates that an aborts_if should be asserted.
/// For callers of a function with such an aborts_if, the negation of the condition becomes
/// an assertion.
pub const CONDITION_ABORT_ASSERT_PROP: &str = "assert";

/// A property which can be attached to any condition to exclude it from verification. The
/// condition will still be type checked.
pub const CONDITION_DEACTIVATED_PROP: &str = "deactivated";

/// A property which can be attached to an aborts_with to indicate that it should act as check
/// whether the function produces exactly the provided number of error codes.
pub const CONDITION_CHECK_ABORT_CODES_PROP: &str = "check";

/// A property that can be attached to a global invariant to indicate that it should be
/// enabled disabled by the disable_invariant_in_body pragma
pub const CONDITION_SUSPENDABLE_PROP: &str = "suspendable";

/// A function which determines whether a property is valid for a given condition kind.
pub fn is_property_valid_for_condition(kind: &ConditionKind, prop: &str) -> bool {
    if matches!(
        prop,
        CONDITION_INJECTED_PROP
            | CONDITION_EXPORT_PROP
            | CONDITION_ABSTRACT_PROP
            | CONDITION_CONCRETE_PROP
            | CONDITION_DEACTIVATED_PROP
    ) {
        // Applicable everywhere.
        return true;
    }
    use crate::ast::ConditionKind::*;
    match kind {
        GlobalInvariant(..) | GlobalInvariantUpdate(..) => {
            matches!(
                prop,
                CONDITION_GLOBAL_PROP | CONDITION_ISOLATED_PROP | CONDITION_SUSPENDABLE_PROP
            )
        }
        SucceedsIf | AbortsIf => matches!(
            prop,
            CONDITION_ABORT_ASSERT_PROP | CONDITION_ABORT_ASSUME_PROP
        ),
        AbortsWith => matches!(prop, CONDITION_CHECK_ABORT_CODES_PROP),
        _ => {
            // every other condition can only take general properties
            false
        }
    }
}
