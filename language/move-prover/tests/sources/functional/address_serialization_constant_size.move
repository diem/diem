// Tests the additional axiom that constrains address serialization to have the same size.
module AddressSerialization {
    use 0x1::LCS;

    /// Serialized representation of address typed move values have the same vector length.
    public fun serialized_addresses_same_len(addr1: &address, addr2: &address): (vector<u8>, vector<u8>) {
        (LCS::to_bytes(addr1), LCS::to_bytes(addr2))
    }
    spec fun serialized_addresses_same_len {
        ensures len(LCS::serialize(addr1)) == len(LCS::serialize(addr2));
        ensures len(result_1) == len(result_2);
    }

    /// Serialized representation of move values do not have the same length in general.
    public fun serialized_move_values_diff_len_incorrect<MoveValue>(mv1: &MoveValue, mv2: &MoveValue): (vector<u8>, vector<u8>) {
        (LCS::to_bytes(mv1), LCS::to_bytes(mv2))
    }
    spec fun serialized_move_values_diff_len_incorrect {
        ensures len(LCS::serialize(mv1)) == len(LCS::serialize(mv2));
        ensures len(result_1) == len(result_2);
    }
}
