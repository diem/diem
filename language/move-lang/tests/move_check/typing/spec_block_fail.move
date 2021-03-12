module 0x8675309::M {
    fun specs_in_fun(x: u64) {
        (spec {}: u64);
        (spec {}: &u64);
        (spec {}: (u64, u64));
    }
}
