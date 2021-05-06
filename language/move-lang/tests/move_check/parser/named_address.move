// All valid positions of named addresses
address M = 0x1;
address M;
address M {}
address M = 0x1 {}

module M::Mod {

    struct S {}

    friend M::M;
    public(friend) fun foo() {}
}

module M::M {
    use M::Mod::foo;

    struct X { s: M::Mod::S }

    fun bar() { foo() }
}
