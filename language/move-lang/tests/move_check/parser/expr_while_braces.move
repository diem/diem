module M {
    f(v: u64) {
        // Braces required for assignment inside "while" expression
        while (v < 10) v = v + 1
    }
}
