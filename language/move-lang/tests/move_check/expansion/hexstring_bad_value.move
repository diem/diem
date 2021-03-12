module 0x8675309::M {
    public fun bad_value(): vector<u8> {
        x"g0"
    }
    public fun odd_length(): vector<u8> {
        x"123"
    }
}
