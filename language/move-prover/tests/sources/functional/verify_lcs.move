// This file is created to verify the native function in the standard LCS module.

module VerifyLCS {
    use 0x1::LCS;

    public fun verify_to_bytes<MoveValue>(v: &MoveValue): vector<u8>
    {
        LCS::to_bytes(v)
    }
    spec fun verify_to_bytes {
        ensures result == LCS::serialize(v);
    }
}
