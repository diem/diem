address 0x2 {

module A {
    use 0x2::B;
    use 0x2::C;
    friend B;

    public fun a() {
        C::c()
    }
}

module B {
    friend 0x2::C;
}

module C {
    public fun c() {}
}

}
