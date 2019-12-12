module M {
    f(v: u64) {
        // Braces are not required for a control expression inside "loop"
        loop if (v < 10) break
    }
}
