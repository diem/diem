module M {
    fun t1() {
        let a = 0;
        let r1 = &mut a;
        let r2 = &mut a;
        *r1 = 1;
        *r2 = 2;
    }
}
