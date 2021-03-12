module 0x8675309::M {
    struct Coin {}
    struct R<T: key>  { r: T }
    struct S<T: drop> has drop { c: T }

    fun t0() {
        R {r:_ } = R { r: 0 };
        S { c: Coin {} };
    }

    fun t1() {
        R {r: R { r: _ } } = R { r: R { r: 0 }};
        S { c: S { c: Coin {} } };
    }
}
