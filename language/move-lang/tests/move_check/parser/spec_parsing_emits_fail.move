module M {
    spec fun with_emits {
        emits _msg;
    }
    fun with_emits<T: copyable>(_guid: vector<u8>, _msg: T, x: u64): u64 {
        x
    }
}
