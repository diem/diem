module M {
    use 0x0::X as X;
    fun foo() {
        X::foo();
    }
}
