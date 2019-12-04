address 0x1:

module A {
    struct S {}
    b(): 0x1::B::S { abort 0 }
}

module B {
    struct S {}
    a(): 0x1::A::S { abort 0 }
    c(): 0x1::C::S { abort 0 }
}

module C {
    struct S {}
    b(): 0x1::B::S { abort 0 }
}


module D {
    struct S {}
    b(): 0x1::B::S { abort 0 }

    e(): 0x1::E::S { abort 0 }
    f(): 0x1::F::S { abort 0 }
}

module E {
    struct S {}
    f(): 0x1::F::S { abort 0 }
}

module F {
    struct S {}
    d(): 0x1::D::S { abort 0 }
}
