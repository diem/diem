module M {
    f(v: u64) {
        // Test an "if" expression missing parenthesis around the condition
        if v < 3 ()
    }
}
