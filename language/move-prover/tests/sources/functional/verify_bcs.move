// This file is created to verify the native function in the standard BCS module.

module VerifyBCS {
    use 0x1::BCS;

    public fun verify_to_bytes<MoveValue>(v: &MoveValue): vector<u8>
    {
        BCS::to_bytes(v)
    }
    spec fun verify_to_bytes {
        ensures result == BCS::serialize(v);
    }
}
