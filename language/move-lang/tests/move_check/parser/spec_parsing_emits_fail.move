module M {
    spec with_emits {
        emits _msg;
    }
    fun with_emits<T: drop>(_guid: vector<u8>, _msg: T, x: u64): u64 {
        x
    }
}
