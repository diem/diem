// Unbound address in all cases
module A::M { // suggests declaration
    use B::M;

    friend C::M;
    friend D::M::foo;

    struct S {
        x: E::M::S,
    }

    fun foo() {
        let x = F::M::S {};
        G::M::foo();
        let c = H::M::C;
        let a = @I; // suggests declaration
    }
}
