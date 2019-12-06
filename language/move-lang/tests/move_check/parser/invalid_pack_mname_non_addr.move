module M {
    struct S {}
    foo() {
        false::M::S { }
    }

    bar() {
        bar()::bar()::M::S { }
    }
}
