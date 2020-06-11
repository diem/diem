address 0x2 {
module N {
    public fun t(): u64 { 0 }
}

module M {
    use 0x2::N::{
        Self as address,
        Self as signer,
        Self as u8,
        Self as u64,
        Self as u128,
        Self as vector,
        Self as move_to_sender,
        Self as move_to,
        Self as move_from,
        Self as borrow_global,
        Self as borrow_global_mut,
        Self as exists,
        Self as freeze,
        Self as assert,
    };

    fun t(): u64 {

        let address = 0;
        let signer = 0;
        let u8 = 0;
        let u64 = 0;
        let u128 = 0;
        let vector = 0;
        let move_to_sender = 0;
        let move_to = 0;
        let move_from = 0;
        let borrow_global = 0;
        let borrow_global_mut = 0;
        let exists = 0;
        let freeze = 0;
        let assert = 0;

        address::t() +
        signer::t() +
        u8::t() +
        u64::t() +
        u128::t() +
        vector::t() +
        move_to_sender::t() +
        move_to::t() +
        move_from::t() +
        borrow_global::t() +
        borrow_global_mut::t() +
        exists::t() +
        freeze::t() +
        assert::t();

        address +
        signer +
        u8 +
        u64 +
        u128 +
        vector +
        move_to_sender +
        move_to +
        move_from +
        borrow_global +
        borrow_global_mut +
        exists +
        freeze +
        assert
    }
}
}
