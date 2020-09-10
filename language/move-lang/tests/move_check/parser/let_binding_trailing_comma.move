module M {
    fun f() {
        let (x1, x2,) = (1, 2); // Test a trailing comma in the let binding
        x1;
        x2;
    }
}
