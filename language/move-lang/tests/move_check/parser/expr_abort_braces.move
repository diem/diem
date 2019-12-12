module M {
    f(v: u64) {
        // Braces are not required for a control expression inside "abort"
        abort if (v == 0) 10 else v
    }
}
