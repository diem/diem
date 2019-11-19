module M {
    foo<T, U>(x: T, y: U) {
    }

    t1() {
        foo<u64, u64>(false, false);
        foo<bool, bool>(0, false);
        foo<bool, bool>(false, 0);
        foo<bool, bool>(0, 0);
    }

    t2<T, U, V>(t: T, u: U, v: V) {
        foo<U, u64>(t, 0);
        foo<V, T>(u, v);
    }

}
