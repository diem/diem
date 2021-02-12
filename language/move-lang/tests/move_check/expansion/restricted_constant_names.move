address 0x2 {

module M {
    // restricted names are invalid due to not starting with A-Z
    const address: u64 = 0;
    const signer: u64 = 0;
    const u8: u64 = 0;
    const u64: u64 = 0;
    const u128: u64 = 0;
    const vector: u64 = 0;
    const move_to: u64 = 0;
    const move_from: u64 = 0;
    const borrow_global: u64 = 0;
    const borrow_global_mut: u64 = 0;
    const exists: u64 = 0;
    const freeze: u64 = 0;
    const assert: u64 = 0;
    // restricted
    const Self: u64 = 0;
}

}

script {
    // restricted names are invalid due to not starting with A-Z
    const address: u64 = 0;
    const signer: u64 = 0;
    const u8: u64 = 0;
    const u64: u64 = 0;
    const u128: u64 = 0;
    const vector: u64 = 0;
    const move_to: u64 = 0;
    const move_from: u64 = 0;
    const borrow_global: u64 = 0;
    const borrow_global_mut: u64 = 0;
    const exists: u64 = 0;
    const freeze: u64 = 0;
    const assert: u64 = 0;
    // restricted
    const Self: u64 = 0;
    fun main() {}
}
