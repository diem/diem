// All valid positions of named addresses
address M {}

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
