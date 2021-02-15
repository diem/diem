address 0x42 {
module A {
    struct A {}
}

module M {
    friend 0x42::A::A;
}
}
