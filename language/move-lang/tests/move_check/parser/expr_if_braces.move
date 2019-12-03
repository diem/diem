module M {
    f(cond: bool) {
        // Braces required for control expression inside "if" expression
        if (cond) if (cond) ()
    }
}
