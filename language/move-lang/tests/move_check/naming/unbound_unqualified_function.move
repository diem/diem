module M {
    foo() {
        bar();
        let x = bar();
        *bar() = 0;
    }
}
