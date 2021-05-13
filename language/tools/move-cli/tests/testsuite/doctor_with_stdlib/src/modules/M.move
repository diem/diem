address 0x2 {
module M {
    use Std::Debug;

    fun f() {
        Debug::print(&7);
    }
}
}
