module AA::Root {
    use AA::A;
    use BA::B;
    use BA::C;

    public fun foo() {
        A::foo();
        B::foo();
        C::foo();
    }
}
