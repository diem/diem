address 0x42 {
module M {

    struct S<T> has drop { f: T }

    fun ex() {
        S<> { f: 0 };
        S<u64, u64> { f: 0 };
    }
}

}
