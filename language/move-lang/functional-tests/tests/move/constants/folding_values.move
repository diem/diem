script {
    const U8: u8 = 0;
    const U64: u64 = 0;
    const U128: u128 = 0;
    const BOOL: bool = false;
    const ADDR: address = 0x42;
    const HEX: vector<u8> = x"42";
    const BYTES: vector<u8> = b"hello";

    fun main() {
        assert(U8 == 0, 42);
        assert(U64 == 0, 42);
        assert(U128 == 0, 42);
        assert(BOOL == false, 42);
        assert(ADDR == 0x42, 42);
        assert(HEX == x"42", 42);
        assert(BYTES == b"hello", 42);
    }
}
