module M {
    public fun bad_value(): vector<u8> {
        x"g0"
    }
    public fun odd_length(): vector<u8> {
        x"123"
    }
}
