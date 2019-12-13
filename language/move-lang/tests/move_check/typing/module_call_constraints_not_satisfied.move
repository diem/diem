module M {
    struct S {}
    resource struct Coin {}
    struct Box<T> {}
    struct Box3<T1, T2, T3> {}

    both<R: resource, C: copyable>(r: R, c: C) {
        abort 0
    }

    cpy<C: copyable>(c: C) {
        abort 0
    }

    rsrc<R: resource>(r: R) {
        abort 0
    }


    t0() {
        both(S{}, Coin{});
        both(0, Coin{})
    }

    t1<R: resource, C: copyable>() {
        both(Box<C> {}, Box<R> {})
    }

    t2<R: resource, C: copyable>() {
        rsrc(Box3<C, C, C> {});

        cpy(Box3<R, C, C> {});
        cpy(Box3<C, R, C> {});
        cpy(Box3<C, C, R> {});

        cpy(Box3<C, R, R> {});
        cpy(Box3<R, C, R> {});
        cpy(Box3<R, R, C> {});

        cpy(Box3<R, R, R> {});
    }

    t3<U, C: copyable>() {
        cpy(Box3<U, C, C> {});
        cpy(Box3<C, U, C> {});
        cpy(Box3<C, C, U> {});

        cpy(Box3<C, U, U> {});
        cpy(Box3<U, C, U> {});
        cpy(Box3<U, U, C> {});

        cpy(Box3<U, U, U> {});
    }
}
