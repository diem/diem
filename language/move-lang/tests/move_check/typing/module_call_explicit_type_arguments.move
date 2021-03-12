module 0x8675309::M {
    fun foo<T, U>(_x: T, _y: U) {
    }

    fun t1() {
        foo<bool, bool>(false, false);
        foo<u64, bool>(0, false);
        foo<bool, u64>(false, 0);
        foo<u64, u64>(0, 0);
    }

    fun t2<T, U, V>(t: T, u: U, v: V) {
        foo<T, u64>(t, 0);
        foo<U, V>(u, v);
    }

}
