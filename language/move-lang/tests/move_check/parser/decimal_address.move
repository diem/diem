// Addresses can be decimal numbers
address 123 {
module M {
    struct S {}
    public fun nop() {}
    public fun bar(): S { S{} }
}
}
module 123::N {
    use 123::M;
    use 123::M::S;
    fun foo(): 123::M::S {
        M::nop();
        (123::M::bar(): S)
    }
}
