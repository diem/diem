module M {
    fun f(v: u64): u64 {
        // Braces or parenthesis are not required for a control expression
        // inside a "return" expression.
        return if (v > 10) 10 else v
    }
}
