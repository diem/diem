module 0x8675309::M {
    fun foo() {
        global_borrow();
        release<u64>();
        sudo(false);
    }
}
