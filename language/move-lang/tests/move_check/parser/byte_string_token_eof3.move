module 0x8675309::M {
    public fun bad_value(): vector<u8> {
        b"\x0";
        b"\x"
    }
}
