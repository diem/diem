address 0x1 {

module A {
    struct S {}
    fun b(): 0x1::B::S { abort 0 }
}

module B {
    struct S {}
    fun a(): 0x1::A::S { abort 0 }
    fun c(): 0x1::C::S { abort 0 }
}

module C {
    struct S {}
    fun b(): 0x1::B::S { abort 0 }
}


module D {
    struct S {}
    fun b(): 0x1::B::S { abort 0 }

    fun e(): 0x1::E::S { abort 0 }
    fun f(): 0x1::F::S { abort 0 }
}

module E {
    struct S {}
    fun f(): 0x1::F::S { abort 0 }
}

module F {
    struct S {}
    fun d(): 0x1::D::S { abort 0 }
}

}
