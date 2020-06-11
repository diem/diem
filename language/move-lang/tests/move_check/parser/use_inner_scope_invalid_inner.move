address 0x2 {
module M {
    fun t() {
        if (cond) use 0x2::M;
    }
}
