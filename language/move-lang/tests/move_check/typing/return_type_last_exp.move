module M {
    resource struct R {}

    fun t0(): () {
        ()
    }

    fun t1(): u64 {
        0
    }

    fun t2(): (u64, bool, R) {
        (0, false, R{})
    }

}
