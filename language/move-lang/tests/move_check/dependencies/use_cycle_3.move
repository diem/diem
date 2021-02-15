address 0x2 {

module A {
    use 0x2::B;

    struct S has drop {}

    public fun s(): S { S{} }

    fun foo(): B::S {
        0x2::B::s()
    }
}

module B {
    use 0x2::C;

    struct S has drop {}

    public fun s(): S { S{} }

    fun foo(): C::S {
        0x2::C::s()
    }
}

module C {
    use 0x2::A;

    struct S has drop {}

    public fun s(): S { S{} }

    fun foo(): A::S {
        0x2::A::s()
    }
}

}

address 0x3 {

module A {

    struct S has drop {}

    public fun s(): S { S{} }

    fun foo() {
        0x3::B::s();
    }
}

module C {

    struct S has drop {}

    public fun s(): S { S{} }

    fun foo() {
        0x3::A::s();
    }
}

module B {

    struct S has drop {}

    public fun s(): S { S{} }

    fun foo() {
        0x3::C::s();
    }
}

}

address 0x4 {

module C {
    struct S has drop {}

    public fun s(): S { S{} }

    fun foo(): 0x4::A::S {
        0x4::A::s()
    }
}

module B {
    struct S has drop {}

    public fun s(): S { S{} }

    fun foo(): 0x4::C::S {
        0x4::C::s()
    }
}

module A {

    struct S{}

    public fun s(): S { S{} }

    fun foo(): 0x4::B::S {
        0x4::B::s()
    }
}

}
