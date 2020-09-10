address 0x2 {
module X {
}

module M {
    use 0x2::X::S;

    struct X { f: S }
}
}
