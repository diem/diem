address 0x2 {

module A {
    friend 0x2::B;
}

module B {
    friend 0x2::C;
    friend 0x2::D;
}

module C {
    friend 0x2::A;
}


module D {
    friend 0x2::E;
}

module E {
    friend 0x2::B;
}

}
