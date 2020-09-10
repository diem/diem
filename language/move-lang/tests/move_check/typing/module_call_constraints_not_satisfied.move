module M {
    struct S {}
    resource struct Coin {}
    struct Box<T> {}
    struct Box3<T1, T2, T3> {}

    fun both<R: resource, C: copyable>(r: R, c: C) {
        abort 0
    }

    fun cpy<C: copyable>(c: C) {
        abort 0
    }

    fun rsrc<R: resource>(r: R) {
        abort 0
    }


    fun t0() {
        both(S{}, Coin{});
        both(0, Coin{})
    }

    fun t1<R: resource, C: copyable>() {
        both(Box<C> {}, Box<R> {})
    }

    fun t2<R: resource, C: copyable>() {
        rsrc(Box3<C, C, C> {});

        cpy(Box3<R, C, C> {});
        cpy(Box3<C, R, C> {});
        cpy(Box3<C, C, R> {});

        cpy(Box3<C, R, R> {});
        cpy(Box3<R, C, R> {});
        cpy(Box3<R, R, C> {});

        cpy(Box3<R, R, R> {});
    }

    fun t3<U, C: copyable>() {
        cpy(Box3<U, C, C> {});
        cpy(Box3<C, U, C> {});
        cpy(Box3<C, C, U> {});

        cpy(Box3<C, U, U> {});
        cpy(Box3<U, C, U> {});
        cpy(Box3<U, U, C> {});

        cpy(Box3<U, U, U> {});
    }
}
