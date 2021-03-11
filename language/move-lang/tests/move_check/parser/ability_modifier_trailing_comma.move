address 0x42 {
module M {
    // invalid trailing comma
    struct Foo has copy, drop, {}
}
}
