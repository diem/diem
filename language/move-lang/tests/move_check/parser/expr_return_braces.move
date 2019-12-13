module M {
    f(v: u64): u64 {
        // Braces are not required for a control expression inside "return"
        return if (v > 10) 10 else v
    }
}
