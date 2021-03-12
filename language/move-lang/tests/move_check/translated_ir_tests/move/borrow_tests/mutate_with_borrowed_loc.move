module 0x8675309::M {
    fun t1() {
        let x = 0;
        let y = &x;
        y;
        y = &x;
        y;
    }
}
