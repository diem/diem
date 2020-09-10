module M {
    resource struct R {}

    fun t0(): () {
        if (true) return () else return ()
    }

    fun t1(): u64 {
        return 0
    }

    fun t2(): (u64, bool, R) {
        while (true) return (0, false, R{});
        abort 0
    }
}
