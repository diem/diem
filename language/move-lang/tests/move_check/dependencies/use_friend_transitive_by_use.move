address 0x2 {

module A {
    use 0x2::B;
    use 0x2::C;
    friend C;

    public fun a() {
        B::b()
    }
}

module B {
    use 0x2::C;
    public fun b() {
        C::c()
    }
}

module C {
    public fun c() {}
}

}
