address 0x42 {
module M {

    struct S<T> has drop, copy { f: T }

    fun ex(s: S<u64>) {
        let S<> { f } = copy s;
        f;
        let S<u64, u64> { f } = copy s;
        f;
    }
}

}
