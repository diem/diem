module A::A {
    use DA::A as DAA;
    use A::A as CAA;

    public fun foo() {
        DAA::foo();
        CAA::foo()
    }
}
