address 0x2 {
module X {
    struct S {}
}

module M {
    use 0x2::X::S;

    struct X { f: S }

    fun foo<S>(x: S): 0x2::X::S {
        x
    }
}
}
