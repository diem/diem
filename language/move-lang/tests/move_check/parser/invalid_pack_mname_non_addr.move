module M {
    struct S {}
    fun foo() {
        false::M::S { }
    }

    fun bar() {
        bar()::bar()::M::S { }
    }
}
