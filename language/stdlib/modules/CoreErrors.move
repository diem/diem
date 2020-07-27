address 0x1 {

/// Module containing common error codes used in the framework.
module CoreErrors {

    /// # General Error Codes

    /// Operation only allowed in genesis.
    public fun NOT_GENESIS(): u64 { 101 }

    /// Operation only allowed outside of genesis.
    public fun NOT_OPERATING(): u64 { 102 }

    /// A resource is attempted to publish twice.
    public fun NOT_INITIALIZED(): u64 { 103 }

    /// A resource is attempted to publish twice.
    public fun ALREADY_INITIALIZED(): u64 { 104 }

    /// Invalid input data has been provided.
    public fun INVALID_INPUT(): u64 { 105 }

    /// An input parameter is out of range.
    public fun OUT_OF_RANGE(): u64 { 106 }

    /// An internal error has happened.
    public fun INTERNAL(): u64 { 107 }

    /// # Errors Codes Related to Expected Signers

    /// Signer expected to be the singleton root.
    public fun NOT_LIBRA_ROOT_ADDRESS(): u64 { 201 }

    /// Signer expected to be the VM.
    public fun NOT_VM_RESERVED_ADDRESS(): u64 { 202 }

    /// # Error Codes Related to Expected Roles

    /// Signer expected to have root role.
    public fun NOT_LIBRA_ROOT_ROLE(): u64 { 301 }

    /// Signer expected to have treasury compliance role.
    public fun NOT_TREASURY_COMPLIANCE_ROLE(): u64 { 302 }

    /// # Custom Error Codes

    /// Custom error codes can be defined on a per-module basis. Each custom error code should provide a public
    /// function to make this code available for scripts and other modules, using naming conventions as here.
    /// Custom error codes should start at the value below. They must not overlap with custom error
    /// codes defined in other modules (please check existing codes when introducing a new one to achieve this.)
    /// Also consider that your custom error codes must be made known to applications running on top of Libra
    /// so they can be converted into appropriate user messages, as these codes become part of the public APIs.
    public fun FIRST_AVAILABLE_CUSTOM_CODE(): u64 { 100000 }
}

}
