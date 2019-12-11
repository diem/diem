module M {
    foo(s: Self::S): Self::S {
        let s = s;
        s
    }

    bar(): Self::S {
        S {}
    }

    baz() {
        S {} = bar();
        Self::S {} = bar();
    }

    bug() {
        let S {} = bar();
        let Self::S {} = bar();
    }
}
