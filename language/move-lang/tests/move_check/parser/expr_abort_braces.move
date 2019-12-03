module M {
    f(v: u64) {
        // Braces required for control expression inside "abort" expression
        abort if (v == 0) 10 else v
    }
}
