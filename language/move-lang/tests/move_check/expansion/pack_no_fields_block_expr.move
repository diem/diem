module M {
    struct S {}
    foo() {
        let s = S { let x = 0; x };
        let s = S { let y = 0; let z = 0; x + foo() };
    }
}
