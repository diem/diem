address 0x0 {

module Hash {
    spec module {
        native define sha2(data: vector<u8>): vector<u8>;
        native define sha3(data: vector<u8>): vector<u8>;
    }
    native public fun sha2_256(data: vector<u8>): vector<u8>;
    native public fun sha3_256(data: vector<u8>): vector<u8>;
}
}
