// This file is created to verify the native function in the standard BCS module.
module 0x42::VerifyBCS {
    use Std::BCS;


    public fun verify_to_bytes<MoveValue>(v: &MoveValue): vector<u8>
    {
        BCS::to_bytes(v)
    }
    spec verify_to_bytes {
        ensures result == BCS::serialize(v);
    }
}
