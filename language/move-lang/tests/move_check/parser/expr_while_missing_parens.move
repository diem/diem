module M {
    f(v: u64) {
        // Test a "while" expression missing parenthesis around the condition
        while v < 3 { v = v + 1 }
    }
}
