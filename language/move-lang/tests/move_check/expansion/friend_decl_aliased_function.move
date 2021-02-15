address 0x42 {
module A {
    public fun a() {}
}

module M {
    use 0x42::A::a;
    friend a;

    public(friend) fun m() {
        a()
    }
}
}
