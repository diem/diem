address 0x2 {

module M {
    use 0x2::X;
    use 0x2::X as X2;

    struct F {
        f: 0x2::X::S,
        g: X::S,
        h: X2::S,
    }

    fun foo(_x: 0x2::X::S, _y: X::S, _z: X2::S): (0x2::X::S, X::S, X2::S) {
        let a : 0x2::X::S = 0x2::X::foo();
        let b : X::S = X::foo();
        let c : X2::S = X2::foo();
        (a, b, c)
    }
}

module X {
    struct S has drop {}
    public fun foo(): S { S{} }
}

}
