address 0x1 {
module X {
    struct S {}
}

module M {
    use 0x1::X::u;
    struct X { f: u }
}
}
