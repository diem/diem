address 0x2 {

module A {
    friend 0x2::B;
}

module B {
    friend 0x2::A;
    friend 0x2::C;
}

module C {
    friend 0x2::B;
}


module D {
    friend 0x2::B;
    friend 0x2::E;
    friend 0x2::F;
}

module E {
    friend 0x2::F;
}

module F {
    friend 0x2::D;
}

}
