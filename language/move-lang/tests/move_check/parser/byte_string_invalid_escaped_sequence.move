module M {
    public fun bad_value1(): vector<u8> {
        b"libr\a"
    }
}
