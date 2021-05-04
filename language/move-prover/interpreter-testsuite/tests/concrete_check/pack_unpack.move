module 0x2::A {
    struct S {
        f1: bool,
        f2: u64,
    }

    #[test]
    public fun pack(): S {
        S { f1: true, f2: 1 }
    }

    #[test]
    public fun unpack(): (bool, u64) {
        let s = S { f1: true, f2: 1 };
        let x1: bool;
        let x2: u64;
        S { f1: x1, f2: x2 } = s;
        (x1, x2)
    }
}
