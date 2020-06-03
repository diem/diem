address 0x1 {
module M {
    fun t() {
        if (cond) use 0x1::M;
    }
}
