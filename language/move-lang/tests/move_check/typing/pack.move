module M {
    struct S { f: u64 }
    struct Nat<T> { f: T }
    resource struct R { s: S, f: u64, n1: Nat<u64>, n2: Nat<S> }

    fun t0() {
        (S { f: 0 } : S);
        let s = S{ f: 0 };
        R { s: _, f: _, n1: _, n2: _ } = (R { s: S{f: 0}, n2: Nat{f: s}, n1: Nat{f: 0}, f: 0, }: R);

        let f = 0;
        let s = S{ f: 0 };
        let n1 = Nat { f };
        let n2 = Nat { f: *&s };
        R { s: _, f: _, n1: _, n2: _ } = (R { s, n2, f, n1 }: R);

        (Nat { f: Nat { f: Nat {f: S { f }}}}: Nat<Nat<Nat<S>>>);
    }
}
