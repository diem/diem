address 0x2 {
module X {
    struct S {}
}

module M {
    use 0x2::X::u;
    struct X { f: u }
}
}
