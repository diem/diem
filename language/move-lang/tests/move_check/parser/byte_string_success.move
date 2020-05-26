module M {
    public fun byte_string(): vector<u8> {
        b"Libra"
    }
    public fun byte_string_with_quotes(): vector<u8> {
        b"\"Libra\""
    }
    public fun byte_string_with_escaped_sequence(): vector<u8> {
        b"Hello\n libra. \n Newline; \r Carriage return; \t Tab; \\ Backslash; \0 Null"
    }
    public fun empty_byte_string(): vector<u8> {
        b""
    }
    public fun byte_string_with_hex(): vector<u8> {
        b"\x00\x01\x02"
    }
}
