module M {
    f(v: u64) {
        // Braces required for control expression inside "loop" expression
        loop if (v < 10) break
    }
}
