module M {
    fun f(v: u64) {
        // Braces or parenthesis are not required for a control expression
        // inside an "abort" expression.
        abort if (v == 0) 10 else v
    }
}
