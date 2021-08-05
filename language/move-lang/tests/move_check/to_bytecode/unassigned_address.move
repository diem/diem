// Tests compilation is stopped for unassigned addresses
// Named addresses don't exist at the bytecode level

address A {
module Ex {}
}

module A::M {
    struct S {}
    friend A::N;
    public(friend) fun foo(_: address): S { S{} }
}

module A::N {
    fun bar(): A::M::S {
        A::M::foo(@A)
    }
}
