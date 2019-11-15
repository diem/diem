module M {
    struct S { f: u64 }
    struct Nat<T> { f: T }
    resource struct R { s: S, f: u64, n1: Nat<u64>, n2: Nat<S> }

    t0(): R {
        (S { f: false } : S);

        let s = S { f: 0 };
        let r = (R {
            s: S{f: 0},
            n2: Nat{f: s},
            n1: Nat{f: 0},
            f: 0
         } : R);
        (R {
            s: r,
            f: false,
            n1: Nat { f: false },
            n2: Nat{ f: r }
        }: R)
    }
}
