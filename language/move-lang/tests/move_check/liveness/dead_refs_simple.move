module M {
    fun t() {
        let x = 0;
        let x_ref = &mut x;
        *x_ref = 0;
        _ = x;
        _ = move x;
    }
}
