// Addresses can be decimal numbers
address 123 {
module M {
    struct S has copy, drop {}
    public fun s(): S { S{} }
    public fun take(_s: S) {}
}
}

script {
    fun main() {
        0x7B::M::take(0x7B::M::s());
        0x7B::M::take(123::M::s());
        123::M::take(0x7B::M::s());
        123::M::take(123::M::s());
    }
}
