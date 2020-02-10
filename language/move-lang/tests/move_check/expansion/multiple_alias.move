address 0x1:

module M {
    use 0x1::X;
    use 0x1::X as X2;

    struct F {
        f: 0x1::X::S,
        g: X::S,
        h: X2::S,
    }

    fun foo(x: 0x1::X::S, y: X::S, z: X2::S): (0x1::X::S, X::S, X2::S) {
        let a : 0x1::X::S = 0x1::X::foo();
        let b : X::S = X::foo();
        let c : X2::S = X2::foo();
        (a, b, c)
    }
}

module X {
    struct S {}
    public fun foo(): S { S{} }
}
