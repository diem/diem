module M {
    use 0x0::X as X;
    foo() {
        X::foo();
    }
}
