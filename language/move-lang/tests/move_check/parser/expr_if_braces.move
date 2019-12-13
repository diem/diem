module M {
    f(cond: bool) {
        // Braces are not required for a control expression inside "if"
        if (cond) if (cond) ()
    }
}
