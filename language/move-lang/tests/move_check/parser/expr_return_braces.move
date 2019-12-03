module M {
    f(v: u64): u64 {
        // Braces required for control expression inside "return" expression
        return if (v > 10) 10 else v
    }
}
