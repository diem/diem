module 0x8675309::M {
    fun f(cond: bool) {
        // Braces or parenthesis are not required for a control expression
        // inside an "if" expression.
        if (cond) { if (cond) () };
        if (cond) ( if (cond) () );
        if (cond) if (cond) ()
    }
}
