module M {
    f(v: u64) {
        // Braces are not required for a control expression inside "while"
        while (v < 10) v = v + 1
    }
}
