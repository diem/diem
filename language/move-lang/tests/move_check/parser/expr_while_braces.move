module 0x8675309::M {
    fun f(v: u64) {
        // Braces or parenthesis are not required for a control expression
        // inside a "while" expression.
        while (v < 10) { v = v + 1 };
        while (v < 10) ( v = v + 1 );
        while (v < 10) v = v + 1
    }
}
