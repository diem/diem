module M {
    fun specs_in_fun(x: u64) {
        (spec {}: u64);
        (spec {}: &u64);
        (spec {}: (u64, u64));
    }
}
