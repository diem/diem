module M {
    fun f(v: u64) {
        // Braces or parenthesis are not required for a control expression
        // inside a "loop" expression.
        loop { if (v < 10) break };
        loop ( if (v < 10) break );
        loop if (v < 10) break
    }
}
