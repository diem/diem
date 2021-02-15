address 0x2 {

module A {
    use 0x2::B;
    friend B;

    public fun a() {
        B::b()
    }
}

module B {
    public fun b() {}
}

}
