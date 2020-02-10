module M {
    fun foo() {
        bar();
        let x = bar();
        *bar() = 0;
    }
}
