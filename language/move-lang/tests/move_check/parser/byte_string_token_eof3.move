module M {
    public fun bad_value(): vector<u8> {
        b"\x0";
        b"\x"
    }
}
