module 0x8675309::M {
    public fun bad_value(): vector<u8> {
        b"diem \xG0"
    }
}
