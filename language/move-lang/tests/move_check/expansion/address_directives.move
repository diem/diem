module M {

}

address 0x1:

module M {
    struct X {}

    public fun x(): X {
        X { }
    }

}

address 0x2:
address 0x3:
address 0x1:
address 0x3:

module M {
    use 0x1::M;

    struct X {}

    public fun x(): X {
        X {}
    }

    public fun both(): (X, M::X) {
        (X { }, M::x())
    }

}

address 0x1:

module M2 {
    use 0x1::M as M1;
    use 0x3::M as M3;

    struct X {}

    public fun x(): (M1::X, X, M3::X) {
        (M1::x(), X {}, M3::x())
    }

}
