// Utility for converting a Move value to its binary representation in LCS (Libra Canonical
// Serialization). LCS is the binary encoding for Move resources and other non-module values
// published on-chain. See https://github.com/libra/libra/tree/master/common/lcs for more
// details on LCS (TODO: link to spec once we have one)

address 0x0:
module LCS {
    spec module {
        native define serialize<MoveValue>(v: &MoveValue): vector<u8>;
    }
    // Return the binary representation of `v` in LCS (Libra Canonical Serialization) format
    native public fun to_bytes<MoveValue>(v: &MoveValue): vector<u8>;
}
