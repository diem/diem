address 0x2 {

module A {
    friend 0x2::B;
}

module B {
    friend 0x2::A;
}

}
