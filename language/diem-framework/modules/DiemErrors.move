address 0x1 {

/// Module defining error codes used in Move aborts throughout the framework.
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
///
/// >TODO: determine what kind of stability guarantees we give about reasons/associated module.
module DiemErrors {
    /// A function to create an error from from a category and a reason.
    fun make(category: u8, reason: u64): u64 {
        (category as u64) + (reason << 8)
    }
    spec make {
        pragma opaque = true;
        ensures [concrete] result == category + (reason << 8);
        aborts_if [abstract] false;
        ensures [abstract] result == category;
    }

    /// The signer of a transaction does not have the expected  role for this operation. Example: a call to a function
    /// which requires the signer to have the role of treasury compliance.
    const REQUIRES_ROLE: u8 = 3;

    public fun requires_role(reason: u64): u64 { make(REQUIRES_ROLE, reason) }
    spec requires_role {
        pragma opaque = true;
        aborts_if false;
        ensures result == REQUIRES_ROLE;
    }
}
}
