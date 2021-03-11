address 0x42 {
module M {
    // incorrect delim for ability modifiers
    struct Foo has copy & drop {}
}
}
