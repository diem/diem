module M {
    resource struct Coin {}
    struct R<T:resource>  { r: T }
    struct S<T:copyable> { c: T }

    t0() {
        R {r:_ } = R { r: 0 };
        S { c: Coin {} };
    }

    t1() {
        R {r: R { r: _ } } = R { r: R { r: 0 }};
        S { c: S { c: Coin {} } };
    }
}
