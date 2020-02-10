address 0x1:

module A {
    struct S {}
    fun b(): 0x1::B::S { abort 0 }
}

module B {
    struct S {}
    fun c(): 0x1::C::S { abort 0 }

    fun d(): 0x1::D::S { abort 0 }
}

module C {
    struct S {}
    fun A(): 0x1::A::S { abort 0 }
}


module D {
    struct S {}
    fun e(): 0x1::E::S { abort 0 }
}

module E {
    struct S {}
    fun b(): 0x1::B::S { abort 0 }
}
