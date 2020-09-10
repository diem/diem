module M {
    struct S {}
    fun foo() {
        false::M::S { }
    }

    fun bar() {
        fun bar()::bar()::M::S { }
    }
}
