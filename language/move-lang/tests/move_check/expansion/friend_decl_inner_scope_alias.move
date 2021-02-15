address 0x42 {
module A {
    public fun a() {}
}

module M {
    friend A;

    public(friend) fun m() {
        use 0x42::A;
        A::a()
    }
}
}
