address 0x2 {

module A {
    struct S {}
    fun b(): 0x2::B::S { abort 0 }
}

module B {
    struct S {}
    fun a(): 0x2::A::S { abort 0 }
    fun c(): 0x2::C::S { abort 0 }
}

module C {
    struct S {}
    fun b(): 0x2::B::S { abort 0 }
}


module D {
    struct S {}
    fun b(): 0x2::B::S { abort 0 }

    fun e(): 0x2::E::S { abort 0 }
    fun f(): 0x2::F::S { abort 0 }
}

module E {
    struct S {}
    fun f(): 0x2::F::S { abort 0 }
}

module F {
    struct S {}
    fun d(): 0x2::D::S { abort 0 }
}

}
