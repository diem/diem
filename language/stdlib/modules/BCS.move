address 0x1 {
/// Utility for converting a Move value to its binary representation in BCS (Binary Canonical
/// Serialization). BCS is the binary encoding for Move resources and other non-module values
/// published on-chain. See https://github.com/diem/diem/tree/master/common/bcs for more
/// details on BCS.
module BCS {
    /// Return the binary representation of `v` in BCS (Binary Canonical Serialization) format
    native public fun to_bytes<MoveValue>(v: &MoveValue): vector<u8>;

    // ==============================
    // Module Specification
    spec module {} // switch to module documentation context

    spec module {
        /// Native function which is defined in the prover's prelude.
        native define serialize<MoveValue>(v: &MoveValue): vector<u8>;
    }
}
}
