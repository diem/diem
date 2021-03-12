module 0x8675309::M {
    fun f(v: u64) {
        let x = *&mut *&v; // Test borrows and dereferences
        x;
    }
    fun annotated(v: u64): u64 {
        (v : u64) // Test an expression annotated with a type
    }
    fun cast(v: u64): u64 {
        (v as u64) // Test a type cast
    }
}
