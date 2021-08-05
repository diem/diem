module 0x8675309::M {
    struct S has copy, drop {}
    struct Coin has key {}
    struct Box<T> has copy, drop { f: T }
    struct Box3<T1, T2, T3> has copy, drop { f1: T1, f2: T2, f3: T3 }

    fun new_box<T>(): Box<T> {
        abort 0
    }

    fun new_box3<T1, T2, T3>(): Box3<T1, T2, T3> {
        abort 0
    }

    fun both<R: key, C: copy>(_r: R, _c: C) {
        abort 0
    }

    fun cpy<C: copy>(_c: C) {
        abort 0
    }

    fun rsrc<R: key>(_r: R) {
        abort 0
    }

    fun t0() {
        both(S{}, Coin{});
        both(0, Coin{})
    }

    fun t1<R: key, C: drop>() {
        both(new_box<C>(), new_box<R>())
    }

    fun t2<R: key, C: drop>() {
        rsrc(new_box3<C, C, C>());

        cpy(new_box3<R, C, C>());
        cpy(new_box3<C, R, C>());
        cpy(new_box3<C, C, R>());

        cpy(new_box3<C, R, R>());
        cpy(new_box3<R, C, R>());
        cpy(new_box3<R, R, C>());

        cpy(new_box3<R, R, R>());
    }

    fun t3<U, C: drop>() {
        cpy(new_box3<U, C, C>());
        cpy(new_box3<C, U, C>());
        cpy(new_box3<C, C, U>());

        cpy(new_box3<C, U, U>());
        cpy(new_box3<U, C, U>());
        cpy(new_box3<U, U, C>());

        cpy(new_box3<U, U, U>());
    }
}
