address 0x1 {

module A {
    use 0x1::B;

    struct S{}

    public fun s(): S { S{} }

    fun foo(): B::S {
        0x1::B::s()
    }
}

module B {
    use 0x1::C;

    struct S{}

    public fun s(): S { S{} }

    fun foo(): C::S {
        0x1::C::s()
    }
}

module C {
    use 0x1::A;

    struct S{}

    public fun s(): S { S{} }

    fun foo(): A::S {
        0x1::A::s()
    }
}

}

address 0x2 {

module A {

    struct S{}

    public fun s(): S { S{} }

    fun foo() {
        0x2::B::s();
    }
}

module C {

    struct S{}

    public fun s(): S { S{} }

    fun foo() {
        0x2::A::s();
    }
}

module B {

    struct S{}

    public fun s(): S { S{} }

    fun foo() {
        0x2::C::s();
    }
}

}

address 0x3 {

module C {
    struct S{}

    public fun s(): S { S{} }

    fun foo(): 0x3::A::S {
        0x3::A::s()
    }
}

module B {
    struct S{}

    public fun s(): S { S{} }

    fun foo(): 0x3::C::S {
        0x3::C::s()
    }
}

module A {

    struct S{}

    public fun s(): S { S{} }

    fun foo(): 0x3::B::S {
        0x3::B::s()
    }
}

}
