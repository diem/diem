address 0x1 {

/// Module defining error codes used in Move aborts throughout the Diem Framework.
///
/// A `u64` error code is constructed from two values:
///
///  1. The *error category* which is encoded in the lower 8 bits of the code. Error categories are
///     declared in this module and are globally unique across the Diem framework. There is a limited
///     fixed set of predefined categories, and the framework is guaranteed to use those consistently.
///
///  2. The *error reason* which is encoded in the remaining 56 bits of the code. The reason is a unique
///     number relative to the module which raised the error and can be used to obtain more information about
///     the error at hand. It is mostly used for diagnosis purposes. Error reasons may change over time as the
///     framework evolves.
module DiemErrors {
    use 0x1::Errors;

    // The numbers assignes to these error categories come from the 0x1::Errors
    // module originally and are preserved for backwards compatibility reasons.

    /// The signer of a transaction does not have the expected  role for this operation. Example: a call to a function
    /// which requires the signer to have the role of treasury compliance.
    const REQUIRES_ROLE: u8 = 3;

    /// The signer of a transaction does not have a required capability.
    const REQUIRES_CAPABILITY: u8 = 4;

    public fun requires_role(reason: u64): u64 { Errors::make(REQUIRES_ROLE, reason) }
    spec requires_role {
        pragma opaque = true;
        aborts_if false;
        ensures result == REQUIRES_ROLE;
    }

    public fun requires_capability(reason: u64): u64 { Errors::make(REQUIRES_CAPABILITY, reason) }
    spec requires_capability {
        pragma opaque = true;
        aborts_if false;
        ensures result == REQUIRES_CAPABILITY;
    }
}
}
