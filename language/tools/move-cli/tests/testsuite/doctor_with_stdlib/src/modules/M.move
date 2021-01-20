address 0x2 {
module M {
    use 0x1::Debug;

    fun f() {
        Debug::print(&7);
    }
}
}
