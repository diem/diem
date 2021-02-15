address 0x42 {
module A {
    fun a() {}
}

module M {
    friend 0x42::A::a;
}
}
