module M {
    use 0x1::X as X;
    fun foo() {
        X::foo();
    }
}
