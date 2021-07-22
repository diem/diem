module A::M {
    use A::T;
    public fun foo() {
        T::foo()
    }
}
