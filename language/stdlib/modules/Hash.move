address 0x1 {

module Hash {
    native public fun sha2_256(data: vector<u8>): vector<u8>;
    native public fun sha3_256(data: vector<u8>): vector<u8>;
}

}
