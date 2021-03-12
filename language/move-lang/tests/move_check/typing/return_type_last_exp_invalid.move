module 0x8675309::M {
    struct R {}

    fun t0(): u64 {
        ()
    }

    fun t1(): () {
        0
    }

    fun t2(): (u64, bool) {
        (0, false, R{})
    }

    fun t3(): (u64, bool, R, bool) {
        (0, false, R{})
    }

    fun t4(): (bool, u64, R) {
        (0, false, R{})
    }
}
