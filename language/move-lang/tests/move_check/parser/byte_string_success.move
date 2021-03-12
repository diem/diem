module 0x8675309::M {
    public fun byte_string(): vector<u8> {
        b"Diem"
    }
    public fun byte_string_with_quotes(): vector<u8> {
        b"\"Diem\""
    }
    public fun byte_string_with_escaped_sequence(): vector<u8> {
        b"Hello\n diem. \n Newline; \r Carriage return; \t Tab; \\ Backslash; \0 Null"
    }
    public fun empty_byte_string(): vector<u8> {
        b""
    }
    public fun byte_string_with_hex(): vector<u8> {
        b"\x00\x01\x02"
    }
    public fun escaped_backslash_before_quote(): vector<u8> {
        b"\\"
    }
}
