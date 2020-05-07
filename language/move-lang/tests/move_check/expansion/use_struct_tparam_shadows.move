address 0x1 {
module X {
    struct S {}
}

module M {
    use 0x1::X::S;

    struct X { f: S }

    fun foo<S>(x: S): 0x1::X::S {
        x
    }
}
}
