module 0x8675309::M {
    // Test a unit return value.
    fun f1(): () { }

    // Test a single type return value.
    fun f2(): u64 { 1 }
    fun f3(): (u64) { 1 }
    fun f4(p: &u64): &u64 { p }

    // Test multiple return values.
    fun f5(): (u64, u64) { (1, 2) }
}
