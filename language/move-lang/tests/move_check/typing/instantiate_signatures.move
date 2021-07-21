address 0x42 {
module M {
    struct S<T: drop> has drop { f: T }
    struct R {}
    fun id<T>(x: T): T { x }

    // Written types with unsatisified constraints
    // Checked in various positions

    struct S1 {
        f1: S<R>,
        f2: S<&u64>,
        f3: &(&u64),
        f4: S<(u64, u64)>,
    }

    fun f1(
        _f1: S<R>,
        _f2: S<&u64>,
        _f3: &(&u64),
        _f4: S<(u64, u64)>,
    ): (
        S<R>,
        S<&u64>,
        &(&u64),
        S<(u64, u64)>,
    ) {
        abort 0
    }

    fun f2() {
        let f1: S<R> = abort 0;
        let f2: S<&u64> = abort 0;
        let f3: &(&u64) = abort 0;
        let f4: S<(u64, u64)> = abort 0;

        id<S<R>>(abort 0);
        id<S<&u64>>(abort 0);
        id<&(&u64)>(abort 0);
        id<S<(u64, u64)>>(abort 0);

        S<S<R>> { f: abort 0 };
        S<S<&u64>> { f: abort 0 };
        S<&(&u64)> { f: abort 0 };
        S<S<(u64, u64)>> { f: abort 0 };
    }
}
}
