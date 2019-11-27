module M {
    foo() {
        let f = 0;
        false::M { f } = 0;

        let f = 0;
        0::M { f } = 0;

        let f = 0;
        foo().M { f } = 0;
    }

}
