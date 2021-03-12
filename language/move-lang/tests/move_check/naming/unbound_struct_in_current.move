module 0x8675309::M {
    fun foo(s: Self::S): Self::S {
        let s = s;
        s
    }

    fun bar(): Self::S {
        S {}
    }

    fun baz() {
        S {} = bar();
        Self::S {} = bar();
    }

    fun bug() {
        let S {} = bar();
        let Self::S {} = bar();
    }
}
