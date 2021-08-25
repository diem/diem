module A::A {
    use DA::A as DAA;
    use CA::A as CAA;

    public fun foo() {
        DAA::foo();
        CAA::foo()
    }
}
